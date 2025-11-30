use crate::{colors::ColorSettings, logger::LogObject};
use is_terminal::IsTerminal;
use serde::ser::Serializer as SerSerializer;
use serde::{self, ser::SerializeMap};
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

    let is_tty = stderr().is_terminal();

    let level_value = if is_tty {
        log.log_level.get_colored_string(&state.color_settings)
    } else {
        log.log_level.as_str().to_string()
    };

    obj.serialize_entry("level", level_value.as_str())?;
    obj.serialize_entry(
        &state.timestamp_key,
        &log.timestamp.format(&state.timestamp_format).to_string(),
    )?;

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
