use std::sync::OnceLock;

use crate::Logger;

pub(crate) static GLOBAL_LOGGER: OnceLock<Logger> = OnceLock::new();

#[doc(hidden)]
pub fn get_global_logger() -> &'static Logger {
    GLOBAL_LOGGER
        .get()
        .expect("Global logger not initialized. Call Logger::init().build() first")
}
