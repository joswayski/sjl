use crate::{colors::ColorSettings, logger::LogObject};
use hashbrown::HashMap;
use is_terminal::IsTerminal;
use serde::{self, Serialize};
use serde_json::Value;
use std::{io::stderr, sync::Arc};

// Note: timestamp is not on here as it can be overridden
pub const RESERVED_FIELD_NAMES: [&str; 4] = ["level", "context", "message", "data"];

#[derive(Serialize)]
pub struct LogOutput<'a> {
    level: &'a str,
    timestamp: String,
    #[serde(flatten)]
    context: &'a HashMap<String, Value>,
    message: &'a Option<String>,
    data: &'a Value,
}

// Thr reason for this is that we don't want to reserialize certain fields
// Specifically the context fields on each log
// So this acts as a cache absically and we Arc it so that it can
// 1. be used in here in the formatter
// 2. be passe down to the logger in .build()
pub struct FormatState {
    pub timestamp_format: String,
    pub timestamp_key: String,
    pub color_settings: ColorSettings,
    pub pretty: bool,
    pub context_fields_pretty: Option<serde_json::Map<String, Value>>,
    pub context_fields_standard: Option<String>,
}

pub fn format_log_line(log: &LogObject, format_state: &Arc<FormatState>) -> String {
    let level_as_str = log.log_level.as_str();
    // Only apply colors if stderr is connected to a terminal (TTY)
    // This prevents ANSI codes from breaking JSON parsing in log aggregators
    let is_tty = stderr().is_terminal();

    let level_str_colored = if is_tty {
        log.log_level
            .get_colored_string(&format_state.color_settings)
    } else {
        // No TTY, use plain text (valid JSON)
        level_as_str.to_string()
    };

    if format_state.pretty {
        // Build a complete JSON object and use serde_json's pretty printer
        let mut output = serde_json::Map::new();
        output.insert("level".to_string(), Value::String(level_as_str.to_string()));
        output.insert(
            format_state.timestamp_key.to_string(),
            Value::String(
                log.timestamp
                    .format(&format_state.timestamp_format)
                    .to_string(),
            ),
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

        // Pretty context fields

        if let Some(ctx) = &format_state.context_fields_pretty {
            for (k, v) in ctx {
                output.insert(k.clone(), v.clone());
            }
        }
        let json_output = Value::Object(output);
        let pretty_json = serde_json::to_string_pretty(&json_output).unwrap();

        // If outputting to a TTY, replace the plain level string with the colored version
        // This makes the output technically not valid JSON (due to ANSI codes),
        // but it displays correctly in terminals and remains valid JSON for log aggregators
        if is_tty {
            pretty_json.replace(
                &format!(r#""level": "{level_as_str}""#),
                &format!(r#""level": "{level_str_colored}""#),
            )
        } else {
            pretty_json
        }
    } else {
        // Original compact format
        let context_part = if let Some(fields) = &format_state.context_fields_standard {
            if fields.is_empty() {
                String::new()
            } else {
                format!(",{}", fields)
            }
        } else {
            String::new()
        };

        log.message.as_ref().map_or_else(
            // when log.message is None
            || {
                if log.data.as_str().is_some() {
                    // No message, data is a string -> use data as the message field
                    format!(
                        r#"{{"level":"{}","{}":"{}","message":"{}"{}}}"#,
                        level_str_colored,
                        &format_state.timestamp_key,
                        log.timestamp.format(&format_state.timestamp_format),
                        log.data.as_str().unwrap(),
                        context_part
                    )
                } else {
                    // No message, data is not a string -> output data field
                    format!(
                        r#"{{"level":"{}","{}":"{}","data":{}{}}}"#,
                        level_str_colored,
                        &format_state.timestamp_key,
                        log.timestamp.format(&format_state.timestamp_format),
                        serde_json::to_string(&log.data).unwrap(),
                        context_part
                    )
                }
            },
            // Message exists -> output both message and data fields
            |msg| {
                format!(
                    r#"{{"level":"{}","{}":"{}","message":"{}","data":{}{}}}"#,
                    level_str_colored,
                    &format_state.timestamp_key,
                    log.timestamp.format(&format_state.timestamp_format),
                    msg,
                    serde_json::to_string(&log.data).unwrap(),
                    context_part
                )
            },
        )
    }
}
