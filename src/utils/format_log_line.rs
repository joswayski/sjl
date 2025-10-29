use crate::{LogLevel, colors::ColorSettings, logger::LogObject};
use hashbrown::HashMap;
use owo_colors::OwoColorize;
use serde::{self, Serialize};
use serde_json::Value;

pub(crate) const RESERVED_FIELD_NAMES: [&str; 5] =
    ["level", "timestamp", "context", "message", "data"];

#[derive(Serialize)]
pub(crate) struct LogOutput<'a> {
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
) -> String {
    let level_str = match log.log_level {
        LogLevel::Debug => "DEBUG".truecolor(
            colors_settings.debug.red,
            colors_settings.debug.green,
            colors_settings.debug.blue,
        ),
        LogLevel::Info => "INFO".truecolor(
            colors_settings.info.red,
            colors_settings.info.green,
            colors_settings.info.blue,
        ),
        LogLevel::Warn => "WARN".truecolor(
            colors_settings.warn.red,
            colors_settings.warn.green,
            colors_settings.warn.blue,
        ),

        LogLevel::Error => "ERROR".truecolor(
            colors_settings.error.red,
            colors_settings.error.green,
            colors_settings.error.blue,
        ),
    };

    // Build context fields as comma-separated JSON key-value pairs
    // Example: "environment": "production", "service": {"name": "api", "version": 1}
    let context_fields = log
        .context
        .iter()
        .map(|(k, v)| {
            // Format each context field as "key": value
            // serde_json::to_string automatically handles proper JSON serialization
            // - Strings become "value"
            // - Numbers become 42
            // - Objects become {"nested": "data"}
            format!(r#""{}": {}"#, k, serde_json::to_string(v).unwrap())
        })
        .collect::<Vec<_>>()
        .join(", ");

    // Prepend a comma if there are context fields, so they merge seamlessly into the JSON
    // If empty: ""
    // If not empty: ,"environment": "production","service": {...}
    let context_part = if context_fields.is_empty() {
        String::new()
    } else {
        format!(",{}", context_fields)
    };

    // Build the final JSON string
    // Format: {"level":"INFO","timestamp":"...","message":"...","data":{...},"context_key":"context_value"}
    if let Some(msg) = &log.message {
        format!(
            r#"{{"level":"{}","timestamp":"{}","message":"{}","data":{}{}}}"#,
            //        ^^                                                  ^^   - Escaped braces for literal { }
            //          ^ Format placeholder for colored level_str
            //                                                      ^^   - Format placeholder for data (no quotes - already JSON)
            //                                                         ^^ - Format placeholder for context_part
            //                                                            ^^  - Escaped closing brace
            level_str,
            log.timestamp.format(timestamp_format),
            msg,
            serde_json::to_string(&log.data).unwrap(), // Already valid JSON
            context_part                               // Either "" or ",key: value,key2: value2"
        )
    } else {
        format!(
            r#"{{"level":"{}","timestamp":"{}","data":{}{}}}"#,
            level_str,
            log.timestamp.format(timestamp_format),
            serde_json::to_string(&log.data).unwrap(),
            context_part
        )
    }
}
