use crate::{
    log_event::LogEvent,
    log_level::LogLevel,
    logger_options::{LOGGER_INITIALIZED, LoggerOptions},
    timestamp::{DEFAULT_TS_FORMAT, FormattedTimestamp},
};
use chrono::{SecondsFormat, Utc};
use serde::Serialize;
use serde_json::{Map, Value};
use std::{
    io::Write,
    mem,
    sync::{
        atomic::Ordering,
        mpsc::{self, Receiver, RecvTimeoutError},
    },
    time::Duration,
};

#[must_use = "Logger does nothing unless you keep it and call log methods like `.info()`"]
pub struct Logger {
    pub(crate) sender: Option<mpsc::Sender<Vec<u8>>>,
    pub(crate) worker: Option<std::thread::JoinHandle<()>>,

    // Options
    pub(crate) min_level: LogLevel,
    pub(crate) timestamp_format: Option<&'static str>,
    pub(crate) context: Map<String, Value>,
    pub(crate) pretty: bool,
}

impl Default for Logger {
    fn default() -> Self {
        LoggerOptions::default().init()
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        // Drop the sender so worker gets Disconnected
        self.sender.take();

        // Wait for thread to flush and exit
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }

        // Let others create
        let _ = LOGGER_INITIALIZED.swap(false, Ordering::SeqCst);
    }
}

impl Logger {
    pub fn builder() -> LoggerOptions {
        LoggerOptions::default()
    }
    pub fn new() -> Self {
        LoggerOptions::default().init()
    }
    pub fn info<CustomData: Serialize>(&self, message: impl AsRef<str>, custom_data: CustomData) {
        self.log(LogLevel::Info, message.as_ref(), custom_data);
    }
    pub fn warn<CustomData: Serialize>(&self, message: impl AsRef<str>, custom_data: CustomData) {
        self.log(LogLevel::Warn, message.as_ref(), custom_data);
    }
    pub fn error<CustomData: Serialize>(&self, message: impl AsRef<str>, custom_data: CustomData) {
        self.log(LogLevel::Error, message.as_ref(), custom_data);
    }
    pub fn debug<CustomData: Serialize>(&self, message: impl AsRef<str>, custom_data: CustomData) {
        self.log(LogLevel::Debug, message.as_ref(), custom_data);
    }

    fn log<CustomData: Serialize>(
        &self,
        log_level: LogLevel,
        message: impl AsRef<str>,
        custom_data: CustomData,
    ) {
        if log_level.severity() < self.min_level.severity() {
            return;
        }

        let timestamp = match self.timestamp_format {
            // Use the provided formatter if available
            Some(fmt) => Utc::now().format(fmt).to_string(),
            None => Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        };

        let data = if mem::size_of::<CustomData>() == 0 {
            None
        } else {
            Some(&custom_data)
        };
        let log_event = LogEvent {
            context: &self.context,
            level: log_level.as_str(),
            // TODO custom timestamp key with custom serialize impl
            timestamp: FormattedTimestamp {
                dt: Utc::now(),
                fmt: self.timestamp_format.unwrap_or(DEFAULT_TS_FORMAT),
            },
            data,
            message: message.as_ref(),
        };

        // TODO buffers
        let mut log_event = match if self.pretty {
            serde_json::to_vec_pretty(&log_event)
        } else {
            serde_json::to_vec(&log_event)
        } {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("Error ocurred converting log event to bytes. Error: {e}");
                return;
            }
        };

        // newline
        log_event.push(b'\n');

        if let Some(sender) = &self.sender {
            let _ = sender.send(log_event);
        }
    }

    pub(crate) fn handle_messages(
        worker: Receiver<Vec<u8>>,
        max_bytes: usize,
        max_messages: u16,
        flush_interval: Duration,
    ) -> std::thread::JoinHandle<()> {
        // Spawn a dedicated thread for logs
        std::thread::spawn(move || {
            let mut batch = Vec::<u8>::with_capacity(max_bytes);
            let mut message_count: u16 = 0;
            loop {
                match worker.recv_timeout(flush_interval) {
                    Ok(log_bytes) => {
                        batch.extend_from_slice(&log_bytes);
                        message_count += 1;

                        // Happy path
                        if Logger::should_flush(&batch, message_count, max_bytes, max_messages) {
                            Logger::flush(&mut batch, &mut message_count);
                        }
                    }
                    // Everything else, flush regardless of what happened
                    Err(RecvTimeoutError::Disconnected) => {
                        Logger::flush(&mut batch, &mut message_count);
                        break;
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        Logger::flush(&mut batch, &mut message_count);
                    }
                }
            }
        })
    }

    fn should_flush(batch: &[u8], message_count: u16, max_bytes: usize, max_messages: u16) -> bool {
        if message_count >= max_messages || batch.len() >= max_bytes {
            return true;
        }

        false
    }
    fn flush(batch: &mut Vec<u8>, message_count: &mut u16) {
        if batch.is_empty() {
            return;
        }

        let mut stdout = std::io::stdout().lock();

        let _ = stdout.write_all(batch);
        let _ = stdout.flush();

        batch.clear();
        *message_count = 0;
    }
}
