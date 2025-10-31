use std::io::{Write, stderr};
use std::sync::{Arc, Mutex};

use chrono::Utc;
use serde::Serialize;
use serde_json::Value;

use crate::logger::LoggerContext;
use crate::{
    colors::ColorSettings,
    constants::{
        DEFAULT_BATCH_DURATION_MS, DEFAULT_BATCH_SIZE, DEFAULT_BUFFER_SIZE,
        DEFAULT_TIMESTAMP_FORMAT,
    },
    utils::format_log_line,
};

use super::{levels::LogLevel, options::LoggerOptions};

#[derive(Serialize)]
pub(crate) struct LogObject {
    pub(crate) log_level: LogLevel,
    pub(crate) data: Value,
    #[serde(skip)] // Don't serialize directly
    pub(crate) timestamp: chrono::DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) message: Option<String>,
    #[serde(skip)] // We will handle this
    pub(crate) context: Arc<LoggerContext>,
}

/// Handles graceful shutdown of the logger worker thread.
///
/// When dropped, this will signal the worker thread to finish processing
/// any remaining logs and wait for it to complete.
pub(crate) struct ShutdownHandle {
    shutdown_sender: Mutex<Option<crossbeam_channel::Sender<()>>>,
    worker_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl ShutdownHandle {
    pub(crate) fn new(
        shutdown_sender: crossbeam_channel::Sender<()>,
        worker_thread: std::thread::JoinHandle<()>,
    ) -> Self {
        Self {
            shutdown_sender: Mutex::new(Some(shutdown_sender)),
            worker_thread: Mutex::new(Some(worker_thread)),
        }
    }

    /// Trigger shutdown and wait for worker thread to finish processing all logs.
    pub(crate) fn shutdown(&self) {
        // Drop the shutdown sender to signal the worker thread
        if let Ok(mut sender) = self.shutdown_sender.lock() {
            sender.take();
        }

        // Wait for worker thread to finish processing
        if let Ok(mut handle) = self.worker_thread.lock() {
            if let Some(thread) = handle.take() {
                let _ = thread.join();
            }
        }
    }
}

impl Drop for ShutdownHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// An async JSON logger with batched writes and colorized output.
///
/// Create a new logger using the builder pattern:
/// ```rust,no_run
/// use sajl::{Logger, LogLevel};
///
/// let logger = Logger::init()
///     .min_level(LogLevel::Info)
///     .batch_size(100)
///     .build();
/// ```
pub struct Logger {
    pub(crate) log_sender: crossbeam_channel::Sender<LogObject>,
    pub(crate) min_level: LogLevel,
    pub(crate) timestamp_format: String,
    pub(crate) color_settings: ColorSettings,
    pub(crate) shutdown_handle: Arc<ShutdownHandle>,
    pub(crate) context: Arc<LoggerContext>,
    pub(crate) pretty: bool,
}

impl Logger {
    /// Initialize a new logger with the builder pattern.
    ///
    /// Returns a [`LoggerOptions`] builder that can be configured with:
    /// - `.min_level()` - Set minimum log level
    /// - `.batch_size()` - Set number of logs per batch
    /// - `.batch_duration_ms()` - Set flush interval
    /// - `.buffer_size()` - Set channel capacity
    /// - `.timestamp_format()` - Set timestamp format
    ///
    /// Call `.build()` to create the logger.
    pub fn init() -> LoggerOptions {
        LoggerOptions {
            buffer_size: DEFAULT_BUFFER_SIZE,
            batch_size: DEFAULT_BATCH_SIZE,
            batch_duration_ms: DEFAULT_BATCH_DURATION_MS,
            min_level: LogLevel::Debug,
            timestamp_format: DEFAULT_TIMESTAMP_FORMAT.to_string(),
            color_settings: ColorSettings::default(),
            context: LoggerContext::new(),
            pretty: false,
        }
    }

    fn log<T: Serialize>(&self, message: Option<String>, data: &T, log_level: LogLevel) {
        if log_level < self.min_level {
            return;
        }
        let value = match serde_json::to_value(data) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to serialize {}", e);
                return;
            }
        };

        let log_object = LogObject {
            log_level,
            data: value,
            message,
            timestamp: Utc::now(),
            context: Arc::clone(&self.context),
        };

        if let Err(err) = self.log_sender.try_send(log_object) {
            // Channel full or disconnected. Write synchronously to avoid loss.
            let mut stderr = stderr().lock();
            match err {
                crossbeam_channel::TrySendError::Full(log) => {
                    let inline = LogObject {
                        log_level: log.log_level,
                        data: log.data,
                        message: log.message,
                        timestamp: Utc::now(),
                        context: Arc::clone(&self.context),
                    };
                    writeln!(
                        stderr,
                        "{}",
                        format_log_line(&inline, &self.timestamp_format, &self.color_settings, self.pretty)
                    )
                    .ok();

                    let warning = LogObject {
                        message: None,
                        log_level:  LogLevel::Warn,
                        data: serde_json::to_value("Logger buffer full - consider increasing the buffer_size! This log bypassed batching.").unwrap(),
                        timestamp: Utc::now(),
                        context: Arc::clone(&self.context),
                    };

                    writeln!(
                        stderr,
                        "{}",
                        format_log_line(&warning, &self.timestamp_format, &self.color_settings, self.pretty)
                    )
                    .ok();
                }
                crossbeam_channel::TrySendError::Disconnected(log) => {
                    let inline = LogObject {
                        log_level: log.log_level,
                        data: log.data,
                        message: log.message,
                        timestamp: Utc::now(),
                        context: Arc::clone(&self.context),
                    };
                    writeln!(
                        stderr,
                        "{}",
                        format_log_line(&inline, &self.timestamp_format, &self.color_settings, self.pretty)
                    )
                    .ok();
                }
            }
        }
    }
    /// Log a message at the INFO level.
    ///
    /// Accepts any type that implements [`serde::Serialize`].
    pub fn info<T: Serialize>(&self, data: &T) {
        self.log(None, data, LogLevel::Info);
    }

    /// Log a message at the ERROR level.
    ///
    /// Accepts any type that implements [`serde::Serialize`].
    pub fn error<T: Serialize>(&self, data: &T) {
        self.log(None, data, LogLevel::Error);
    }

    /// Log a message at the WARN level.
    ///
    /// Accepts any type that implements [`serde::Serialize`].
    pub fn warn<T: Serialize>(&self, data: &T) {
        self.log(None, data, LogLevel::Warn);
    }

    /// Log a message at the DEBUG level.
    ///
    /// Accepts any type that implements [`serde::Serialize`].
    pub fn debug<T: Serialize>(&self, data: &T) {
        self.log(None, data, LogLevel::Debug);
    }

    pub fn __log_with_message<T: Serialize>(
        &self,
        message: Option<&str>,
        data: &T,
        level: LogLevel,
    ) {
        let owned_message = message.map(|s| s.to_string());
        self.log(owned_message, data, level)
    }
}
