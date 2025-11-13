use std::sync::OnceLock;

use crate::Logger;

pub static GLOBAL_LOGGER: OnceLock<Logger> = OnceLock::new();

#[doc(hidden)]
pub fn get_global_logger() -> &'static Logger {
    GLOBAL_LOGGER
        .get()
        .expect("Global logger not initialized. Call Logger::init().build() first")
}

/// Trigger shutdown of the global logger.
/// This is automatically called at program exit via an atexit handler.
pub fn shutdown_global_logger() {
    if let Some(logger) = GLOBAL_LOGGER.get() {
        logger.shutdown_handle.shutdown();
    }
}
