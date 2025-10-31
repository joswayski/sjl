use crate::{LogLevel, colors::ColorSettings, logger::LogObject};
use hashbrown::HashMap;
use is_terminal::IsTerminal;
use owo_colors::OwoColorize;
use serde::{self, Serialize};
use serde_json::Value;
use std::io::stderr;

pub(crate) const RESERVED_FIELD_NAMES: [&str; 5] =
    ["level", "timestamp", "context", "message", "data"];

#[derive(Serialize)]
pub struct LogOutput<'a> {
    level: &'a str,
    timestamp: String,
    #[serde(flatten)]
    context: &'a HashMap<String, Value>,
    message: &'a Option<String>,
    data: &'a Value,
}

pub fn format_log_line(
    log: &LogObject,
    timestamp_format: &str,
    colors_settings: &ColorSettings,
    pretty: bool,
) -> String {
    let level_str_plain = match log.log_level {
        LogLevel::Debug => "DEBUG",
        LogLevel::Info => "INFO",
        LogLevel::Warn => "WARN",
        LogLevel::Error => "ERROR",
    };

    // Only apply colors if stderr is connected to a terminal (TTY)
    // This prevents ANSI codes from breaking JSON parsing in log aggregators
    let is_tty = stderr().is_terminal();

    let level_str_colored = if is_tty {
        match log.log_level {
            LogLevel::Debug => "DEBUG"
                .truecolor(
                    colors_settings.debug.red,
                    colors_settings.debug.green,
                    colors_settings.debug.blue,
                )
                .to_string(),
            LogLevel::Info => "INFO"
                .truecolor(
                    colors_settings.info.red,
                    colors_settings.info.green,
                    colors_settings.info.blue,
                )
                .to_string(),
            LogLevel::Warn => "WARN"
                .truecolor(
                    colors_settings.warn.red,
                    colors_settings.warn.green,
                    colors_settings.warn.blue,
                )
                .to_string(),
            LogLevel::Error => "ERROR"
                .truecolor(
                    colors_settings.error.red,
                    colors_settings.error.green,
                    colors_settings.error.blue,
                )
                .to_string(),
        }
    } else {
        // No TTY, use plain text (valid JSON)
        level_str_plain.to_string()
    };

    if pretty {
        // Build a complete JSON object and use serde_json's pretty printer
        let mut output = serde_json::Map::new();
        output.insert(
            "level".to_string(),
            Value::String(level_str_plain.to_string()),
        );
        output.insert(
            "timestamp".to_string(),
            Value::String(log.timestamp.format(timestamp_format).to_string()),
        );

        if let Some(msg) = &log.message {
            output.insert("message".to_string(), Value::String(msg.clone()));
        }

        // If the "data" passed in is a string, treat it as a message and leave it at the top
        if log.data.as_str().is_some() && log.message.is_none() {
            output.insert("message".to_string(), log.data.clone());
        } else {
            output.insert("data".to_string(), log.data.clone());
        }

        // Add context fields
        for (k, v) in log.context.iter() {
            output.insert(k.clone(), v.clone());
        }

        let json_output = Value::Object(output);
        let pretty_json = serde_json::to_string_pretty(&json_output).unwrap();

        // If outputting to a TTY, replace the plain level string with the colored version
        // This makes the output technically not valid JSON (due to ANSI codes),
        // but it displays correctly in terminals and remains valid JSON for log aggregators
        if is_tty {
            pretty_json.replace(
                &format!(r#""level": "{}""#, level_str_plain),
                &format!(r#""level": "{}""#, level_str_colored),
            )
        } else {
            pretty_json
        }
    } else {
        // Original compact format
        // Build context fields as comma-separated JSON key-value pairs
        let context_fields = log
            .context
            .iter()
            .map(|(k, v)| format!(r#""{}": {}"#, k, serde_json::to_string(v).unwrap()))
            .collect::<Vec<_>>()
            .join(", ");

        let context_part = if context_fields.is_empty() {
            String::new()
        } else {
            format!(",{}", context_fields)
        };

        if let Some(msg) = &log.message {
            format!(
                r#"{{"level":"{}","timestamp":"{}","message":"{}","data":{}{}}}"#,
                level_str_colored,
                log.timestamp.format(timestamp_format),
                msg,
                serde_json::to_string(&log.data).unwrap(),
                context_part
            )
        } else {
            // If the data is actually a string, treat it that way at the top
            if log.data.as_str().is_some() && log.message.is_none() {
                format!(
                    r#"{{"level":"{}","timestamp":"{}","message":"{}"{}}}"#,
                    level_str_colored,
                    log.timestamp.format(timestamp_format),
                    log.data.as_str().unwrap(),
                    context_part
                )
            } else {
                format!(
                    r#"{{"level":"{}","timestamp":"{}","data":{}{}}}"#,
                    level_str_colored,
                    log.timestamp.format(timestamp_format),
                    serde_json::to_string(&log.data).unwrap(),
                    context_part
                )
            }
        }
    }
}
