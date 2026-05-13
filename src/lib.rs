use std::borrow::Cow;

use chrono::{SecondsFormat, Utc};
use serde_json::Value;

pub struct Logger {}

enum LogLevel {
    Info,
    Debug,
    Warn,
    Error,
}

impl Logger {
    pub fn new() -> Logger {
        Logger {}
    }

    pub fn info(&self) {
        self.log(LogLevel::Info, Some("test data here".into()), None);
    }

    fn log(&self, log_level: LogLevel, message: Option<String>, data: Option<Value>) {
        // https://docs.rs/chrono/latest/chrono/#formatting-and-parsing & https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers
        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let final_message = format!("{{\"timestamp\": \"{}\"}}", timestamp);

        // let final_message = format!("{{\"message\": \"{}\"}}", message);

        // if Some(message) {}

        println!("{}", final_message)
    }
}
