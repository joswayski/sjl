use crate::{colors::ColorSettings, logger::LogObject};
use is_terminal::IsTerminal;
use serde::ser::{SerializeMap, Serializer as _};
use serde_json::{
    Serializer, Value,
    ser::{CompactFormatter, Formatter, PrettyFormatter},
};
use std::io::{self, Write, stderr};

// Note: timestamp is not on here as it can be overridden
pub const RESERVED_FIELD_NAMES: [&str; 4] = ["level", "context", "message", "data"];

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
    pub context_fields: Vec<(String, Value)>,
}

pub fn write_log_line<W: Write>(
    mut writer: W,
    log: &LogObject,
    state: &FormatState,
) -> io::Result<()> {
    if stderr().is_terminal() {
        let colored = format_with_colors(log, state);
        writer.write_all(colored.as_bytes())?;
        writer.write_all(b"\n")
    } else {
        if state.pretty {
            let mut serializer =
                Serializer::with_formatter(&mut writer, PrettyFormatter::with_indent(b"  "));
            write_one_log(&mut serializer, log, state)?;
        } else {
            let mut serializer = Serializer::with_formatter(&mut writer, CompactFormatter);
            write_one_log(&mut serializer, log, state)?;
        }

        writer.write_all(b"\n")
    }
}

fn write_one_log<W, F>(
    serializer: &mut serde_json::Serializer<W, F>,
    log: &LogObject,
    state: &FormatState,
) -> Result<(), serde_json::Error>
where
    W: Write,
    F: Formatter,
{
    let mut obj = serializer.serialize_map(None)?;

    obj.serialize_entry("level", log.log_level.as_str())?;
    let timestamp = log.timestamp.format(&state.timestamp_format).to_string();
    obj.serialize_entry(&state.timestamp_key, &timestamp)?;

    if let Some(msg) = &log.message {
        obj.serialize_entry("message", msg)?;
    }

    if log.message.is_none() && log.data.as_str().is_some() {
        obj.serialize_entry("message", log.data.as_str().unwrap())?;
    } else {
        obj.serialize_entry("data", &log.data)?;
    }

    for (k, v) in &state.context_fields {
        obj.serialize_entry(k, v)?;
    }

    obj.end()
}

fn format_with_colors(log: &LogObject, state: &FormatState) -> String {
    let level_plain = log.log_level.as_str();
    let level_colored = log.log_level.get_colored_string(&state.color_settings);

    if state.pretty {
        let mut output = serde_json::Map::new();
        output.insert("level".to_string(), Value::String(level_plain.to_string()));
        output.insert(
            state.timestamp_key.clone(),
            Value::String(log.timestamp.format(&state.timestamp_format).to_string()),
        );

        if let Some(msg) = &log.message {
            output.insert("message".to_string(), Value::String(msg.clone()));
        }

        if log.message.is_none() && log.data.as_str().is_some() {
            output.insert("message".to_string(), log.data.clone());
        } else {
            output.insert("data".to_string(), log.data.clone());
        }

        for (k, v) in &state.context_fields {
            output.insert(k.clone(), v.clone());
        }

        let json_output = Value::Object(output);
        let pretty_json = serde_json::to_string_pretty(&json_output).unwrap();

        pretty_json.replace(
            &format!(r#""level": "{}""#, level_plain),
            &format!(r#""level": "{}""#, level_colored),
        )
    } else {
        let context_fields = state
            .context_fields
            .iter()
            .map(|(k, v)| format!(r#""{}": {}"#, k, serde_json::to_string(v).unwrap()))
            .collect::<Vec<_>>()
            .join(", ");

        let context_part = if context_fields.is_empty() {
            String::new()
        } else {
            format!(",{context_fields}")
        };

        log.message.as_ref().map_or_else(
            || {
                if log.data.as_str().is_some() {
                    format!(
                        r#"{{"level":"{}","{}":"{}","message":"{}"{}}}"#,
                        level_colored,
                        state.timestamp_key,
                        log.timestamp.format(&state.timestamp_format),
                        log.data.as_str().unwrap(),
                        context_part
                    )
                } else {
                    format!(
                        r#"{{"level":"{}","{}":"{}","data":{}{}}}"#,
                        level_colored,
                        state.timestamp_key,
                        log.timestamp.format(&state.timestamp_format),
                        serde_json::to_string(&log.data).unwrap(),
                        context_part
                    )
                }
            },
            |msg| {
                format!(
                    r#"{{"level":"{}","{}":"{}","message":"{}","data":{}{}}}"#,
                    level_colored,
                    state.timestamp_key,
                    log.timestamp.format(&state.timestamp_format),
                    msg,
                    serde_json::to_string(&log.data).unwrap(),
                    context_part
                )
            },
        )
    }
}
