use crate::{LoggerOptions, log_event::LogEvent, log_level::LogLevel};
use chrono::{SecondsFormat, Utc};
use serde::Serialize;
use serde_json::{Map, Value};
use std::{
    io::Write,
    sync::mpsc::{self, RecvTimeoutError},
    time::Duration,
};

#[must_use = "Logger does nothing unless you keep it and call log methods like `.info()`"]
pub struct Logger {
    pub(crate) context: Map<String, Value>,
    pub(crate) sender: mpsc::Sender<Vec<u8>>,
}

impl Default for Logger {
    fn default() -> Self {
        LoggerOptions::default().init()
    }
}

impl Logger {
    pub fn info<Data: Serialize>(&self, message: impl AsRef<str>, data: Data) {
        self.log(LogLevel::Info, message.as_ref(), data);
    }
    pub fn warn<Data: Serialize>(&self, message: impl AsRef<str>, data: Data) {
        self.log(LogLevel::Warn, message.as_ref(), data);
    }
    pub fn error<Data: Serialize>(&self, message: impl AsRef<str>, data: Data) {
        self.log(LogLevel::Error, message.as_ref(), data);
    }
    pub fn debug<Data: Serialize>(&self, message: impl AsRef<str>, data: Data) {
        self.log(LogLevel::Debug, message.as_ref(), data);
    }

    fn log<Data: Serialize>(&self, log_level: LogLevel, message: impl AsRef<str>, data: Data) {
        let timestamp = Utc::now()
            // https://docs.rs/chrono/latest/chrono/#formatting-and-parsing &
            // https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers
            .to_rfc3339_opts(SecondsFormat::Millis, true);

        let log_event = LogEvent {
            context: &self.context,
            level: log_level.as_str(),
            timestamp: &timestamp,
            data: Some(&data),
            message: message.as_ref(),
        };

        let mut log_event = serde_json::to_vec(&log_event).unwrap(); // todo
        log_event.push(b'\n');

        let _ = self.sender.send(log_event);
    }

    pub(crate) fn handle_messages(
        receiver: mpsc::Receiver<Vec<u8>>,
        max_bytes: usize,
        max_messages: u16,
        flush_interval: Duration,
    ) {
        // Spawn a dedicated thread for logs
        std::thread::spawn(move || {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();

            let mut batch = Vec::<u8>::with_capacity(max_bytes);
            let mut message_count: u16 = 0;
            loop {
                match receiver.recv_timeout(flush_interval) {
                    Ok(log_bytes) => {
                        batch.extend_from_slice(&log_bytes);
                        message_count += 1;

                        if Logger::should_flush(&batch, message_count, max_bytes, max_messages) {
                            Logger::flush(&mut stdout, &mut batch, &mut message_count);
                        }
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        Logger::flush(&mut stdout, &mut batch, &mut message_count);
                        break;
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        Logger::flush(&mut stdout, &mut batch, &mut message_count);
                    }
                }
            }
        });
    }

    fn should_flush(batch: &[u8], message_count: u16, max_bytes: usize, max_messages: u16) -> bool {
        if message_count >= max_messages || batch.len() >= max_bytes {
            return true;
        }

        false
    }
    fn flush(writer: &mut impl Write, batch: &mut Vec<u8>, message_count: &mut u16) {
        if batch.is_empty() {
            return;
        }

        let _ = writer.write_all(batch);
        let _ = writer.flush();

        batch.clear();
        *message_count = 0;
    }
}
