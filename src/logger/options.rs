use serde_json::Value;
use std::{sync::Arc, time::Duration};

use crate::{
    LogLevel, Logger, RGB,
    colors::ColorSettings,
    globals::GLOBAL_LOGGER,
    logger::{LogObject, LoggerContext, logger::ShutdownHandle},
    utils::{RESERVED_FIELD_NAMES, flush_batch},
};

/// Builder for configuring a [`Logger`] instance.
///
/// Created by calling [`Logger::init()`] and finalized with [`.build()`](LoggerOptions::build).
pub struct LoggerOptions {
    pub(crate) buffer_size: usize,
    pub(crate) batch_size: usize,
    pub(crate) batch_duration_ms: u64,
    pub(crate) min_level: LogLevel,
    pub(crate) timestamp_format: String,
    pub(crate) color_settings: ColorSettings,
    pub(crate) context: LoggerContext,
    pub(crate) pretty: bool,
}

impl LoggerOptions {
    /// The lowest logging level to print
    ///
    /// Example: [`LogLevel::Info`] will skip Debug logs and show Info, Warning, and Error only
    ///
    /// Default is [`LogLevel::Debug`]
    pub fn min_level(mut self, log_level: LogLevel) -> Self {
        self.min_level = log_level;
        self
    }

    /// How many messages to send down the channel before
    /// messages start to be dropped.
    ///
    /// Default is [`DEFAULT_BUFFER_SIZE`] - 1024
    ///
    pub fn buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = buffer_size;
        self
    }

    /// How many log messages to batch
    ///
    /// Default is [`DEFAULT_BATCH_SIZE`] - 50
    pub fn batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }

    /// For how long to batch messages for
    ///
    /// Default is [`DEFAULT_BATCH_DURATION_MS`] - 50ms
    pub fn batch_duration_ms(mut self, batch_duration_ms: u64) -> Self {
        self.batch_duration_ms = batch_duration_ms;
        self
    }

    /// Formats the combined date and time per the specified format string.
    /// See the [chrono::format::strftime](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) module for the supported escape sequences.
    /// Default is [`DEFAULT_TIMESTAMP_FORMAT`] - "%Y-%m-%dT%H:%M:%S%.3fZ" which outputs: 2025-10-26T22:04:29.412Z
    pub fn timestamp_format(mut self, timestamp_format: impl Into<String>) -> Self {
        self.timestamp_format = timestamp_format.into();
        self
    }

    /// Sets the debug color using [`RGB`]
    pub fn debug_color(mut self, color: RGB) -> Self {
        self.color_settings.debug = color;
        self
    }

    /// Sets the info color using [`RGB`]
    pub fn info_color(mut self, color: RGB) -> Self {
        self.color_settings.info = color;
        self
    }

    /// Sets the warn color using [`RGB`]
    pub fn warn_color(mut self, color: RGB) -> Self {
        self.color_settings.warn = color;
        self
    }

    /// Sets the error color using [`RGB`]
    pub fn error_color(mut self, color: RGB) -> Self {
        self.color_settings.error = color;
        self
    }

    /// Sets global context for every log message
    /// For example, environment or service-name
    pub fn context(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        let key_string = key.into();

        match key_string.as_str() {
            "level" | "timestamp" | "context" | "data" | "message" => {
                panic!(
                    "Cannot use {} as a context key - it's a reservd field name. Reserved fields: {}",
                    key_string,
                    RESERVED_FIELD_NAMES.join(", ")
                )
            }
            _ => {}
        }
        self.context.insert(key_string, value.into());
        self
    }

    /// Enables pretty-printing of JSON output with indentation and newlines.
    ///
    /// When enabled, logs will be formatted across multiple lines for easier reading.
    /// This is useful for development but should typically be disabled in production
    /// for log aggregation systems that expect one log per line.
    ///
    /// Note: Colors will still be applied to the log level, but the output will
    /// contain ANSI escape codes that may not parse as valid JSON.
    ///
    /// Default is `false` (compact, single-line output)
    pub fn pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }
    /// Build and initialize the logger.
    ///
    /// This spawns a background task that handles batching and writing logs.
    /// The logger is ready to use immediately after calling this method.
    ///
    /// When the program exits, the logger will automatically flush all remaining
    /// logs before shutting down.
    pub fn build(self) -> &'static Logger {
        // If already initialized, return it
        if let Some(logger) = GLOBAL_LOGGER.get() {
            eprintln!(
                "WARNING - LOGGER ALREADY INITIALIZED! ANY NEW SETTINGS WILL NOT BE APPLIED."
            );
            return logger;
        }

        let (log_sender, log_receiver) = crossbeam_channel::bounded::<LogObject>(self.buffer_size);
        let (shutdown_sender, shutdown_receiver) = crossbeam_channel::bounded::<()>(1);

        // Move configuration into the worker thread
        let timestamp_format = self.timestamp_format.clone();
        let colors = self.color_settings;
        let batch_size = self.batch_size;
        let batch_duration = Duration::from_millis(self.batch_duration_ms);
        let pretty = self.pretty;

        let worker_thread = std::thread::spawn(move || {
            let mut batch = Vec::<LogObject>::with_capacity(batch_size);
            let mut deadline = crossbeam_channel::after(batch_duration);

            loop {
                crossbeam_channel::select! {
                    recv(log_receiver) -> msg => match msg {
                        Ok(log) => {
                            batch.push(log);
                            if batch.len() >= batch_size {
                                flush_batch(&batch, &timestamp_format, &colors, pretty);
                                batch.clear();
                                deadline = crossbeam_channel::after(batch_duration);
                            }
                        }
                        Err(_) => {
                            // Sender disconnected, flush remaining logs and exit
                            if !batch.is_empty() {
                                flush_batch(&batch, &timestamp_format, &colors, pretty);
                            }
                            break;
                        }
                    },

                    recv(deadline) -> _ => {
                        if !batch.is_empty() {
                            flush_batch(&batch, &timestamp_format, &colors, pretty);
                            batch.clear();
                        }
                        deadline = crossbeam_channel::after(batch_duration);
                    },

                    recv(shutdown_receiver) -> _ => {
                        // Shutdown signal received - drain all remaining logs
                        // First, drop our receiver handle to stop receiving new messages
                        drop(shutdown_receiver);

                        // Drain any remaining messages in the channel
                        while let Ok(log) = log_receiver.try_recv() {
                            batch.push(log);
                            if batch.len() >= batch_size {
                                flush_batch(&batch, &timestamp_format, &colors, pretty);
                                batch.clear();
                            }
                        }

                        // Flush final batch
                        if !batch.is_empty() {
                            flush_batch(&batch, &timestamp_format, &colors, pretty);
                        }
                        break;
                    }
                }
            }
        });

        let shutdown_handle = Arc::new(ShutdownHandle::new(shutdown_sender, worker_thread));

        let logger = Logger {
            log_sender,
            min_level: self.min_level,
            timestamp_format: self.timestamp_format,
            color_settings: colors,
            shutdown_handle,
            context: Arc::new(self.context),
            pretty: self.pretty,
        };

        let logger_ref = match GLOBAL_LOGGER.set(logger) {
            Ok(_) => {
                // Register atexit handler to ensure logs are flushed on shutdown
                extern "C" fn shutdown_handler() {
                    crate::globals::shutdown_global_logger();
                }
                unsafe {
                    libc::atexit(shutdown_handler);
                }
                GLOBAL_LOGGER.get().unwrap()
            }
            // Incase of a race condition, return the existing one
            Err(_) => GLOBAL_LOGGER.get().unwrap(),
        };

        logger_ref
    }
}
