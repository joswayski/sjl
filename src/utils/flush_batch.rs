use std::{io::stderr, sync::Arc};

use crate::{
    logger::LogObject,
    utils::write_log_line::{FormatState, write_log_line},
};

pub fn flush_batch(batch: &[LogObject], format_state: &Arc<FormatState>) {
    // Lock once for the whole batch
    let mut stderr = stderr().lock();
    for log in batch {
        if let Err(err) = write_log_line(&mut stderr, log, format_state) {
            eprintln!("failed to write log: {err}")
        }
    }
}
