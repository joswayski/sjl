use std::{
    io::{Write, stderr},
    sync::Arc,
};

use crate::{
    logger::LogObject,
    utils::format_log_line::{FormatState, format_log_line},
};

pub fn flush_batch(batch: &[LogObject], format_state: &Arc<FormatState>) {
    // Lock once for the whole batch
    let mut stderr = stderr().lock();
    for log in batch {
        writeln!(stderr, "{}", format_log_line(log, format_state)).ok();
    }
}
