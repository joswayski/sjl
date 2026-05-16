use crate::{log_event::LogEvent, log_level::LogLevel};
use chrono::{SecondsFormat, Utc};
use serde::Serialize;
use serde_json::{Map, Value, json};
use std::io::Write;

#[must_use = "Logger does nothing unless you keep it and call log methods like `.info()`"]
pub struct Logger {
    pub(crate) context: Map<String, Value>,
}

impl Default for Logger {
    fn default() -> Self {
        Logger {
            context: Map::new(),
        }
    }
}

impl Logger {
    pub fn info<Data: Serialize>(&self, message: impl AsRef<str>, data: Data) {
        self.log(LogLevel::Info, message.as_ref(), data);
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

        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();

        let _ = serde_json::to_writer(&mut stdout, &log_event);
        let _ = writeln!(stdout);
    }
}
