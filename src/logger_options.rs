use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, RecvTimeoutError},
    },
    time::Duration,
};

use serde::Serialize;
use serde_json::{Map, Value};

use crate::Logger;

pub static LOGGER_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[must_use = "LoggerOptions does nothing until you call `.init()`"]
pub struct LoggerOptions {
    pub(crate) context: Map<String, Value>,
    /// How many bytes to buffer before flushing. Default is 64kb
    pub(crate) max_bytes: usize,

    /// How many messages to hold in memory before flushing. Default is 100
    pub(crate) max_messages: u16,

    /// How long to wait before flushing if either max_bytes or max_messages are not past their thresholds.
    /// Default is 1 second
    pub(crate) flush_interval: Duration,
}
impl Default for LoggerOptions {
    fn default() -> Self {
        LoggerOptions {
            context: Map::new(),
            max_bytes: 64 * 1024,
            max_messages: 100,
            flush_interval: Duration::from_secs(1),
        }
    }
}

impl LoggerOptions {
    #[must_use = "call `.init()` to create a Logger"]
    pub fn context<V: Serialize>(mut self, key: impl Into<String>, value: V) -> Self {
        let key = key.into();
        match serde_json::to_value(value) {
            // If it's serializable to an object, all is good
            Ok(value) => {
                self.context.insert(key, value);
            }
            // If we failed to parse, return an error
            Err(serialize_error) => {
                eprintln!(
                    "Error serializing context value for key '{}'. It will not be included. \nError: {}",
                    key, serialize_error
                );
            }
        };

        self
    }

    #[must_use = "Logger must be kept to write logs. For example: logger.info()"]
    pub fn init(self) -> Logger {
        if LOGGER_INITIALIZED.swap(true, Ordering::SeqCst) {
            panic!("Logger already initialized! Only call .init() once");
        }

        let (sender, worker) = mpsc::channel::<Vec<u8>>();

        // Run in background
        let worker = Logger::handle_messages(
            worker,
            self.max_bytes,
            self.max_messages,
            self.flush_interval,
        );

        let logger = Logger {
            context: self.context,
            sender: Some(sender),
            worker: Some(worker),
        };

        logger
    }
}
