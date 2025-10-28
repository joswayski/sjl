use owo_colors::OwoColorize;

use crate::{LogLevel, colors::ColorSettings, logger::LogObject};

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

    if let Some(msg) = &log.message {
        format!(
            r#"{{"level":"{}","timestamp":"{}", "message": "{}","data":{}}}"#,
            level_str,
            // Serialize once on flush
            log.timestamp.format(timestamp_format),
            msg,
            serde_json::to_string(&log.data).unwrap()
        )
    } else {
        format!(
            r#"{{"level":"{}","timestamp":"{}","data":{}}}"#,
            level_str,
            // Serialize once on flush
            log.timestamp.format(timestamp_format),
            serde_json::to_string(&log.data).unwrap()
        )
    }
}
