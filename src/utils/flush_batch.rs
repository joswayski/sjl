use std::io::{Write, stderr};

use crate::{colors::ColorSettings, logger::LogObject, utils::format_log_line};

pub fn flush_batch(
    batch: &[LogObject],
    timestamp_format: &str,
    timestamp_key: &str,
    color_settings: &ColorSettings,
    pretty: bool,
) {
    // Lock once for the whole batch
    let mut stderr = stderr().lock();
    for log in batch {
        writeln!(
            stderr,
            "{}",
            format_log_line(log, timestamp_format, timestamp_key, color_settings, pretty)
        )
        .ok();
    }
}
