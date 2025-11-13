#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
mod colors;
mod constants;
mod globals;
mod logger;
mod macros;
mod utils;

pub use colors::RGB;
pub use globals::get_global_logger;
pub use logger::{LogLevel, Logger, LoggerOptions};
