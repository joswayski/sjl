use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self},
    },
    time::Duration,
};

use crossbeam_queue::ArrayQueue;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::{Logger, log_level::LogLevel};

pub static LOGGER_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[must_use = "LoggerOptions does nothing until you call `.init()`"]
pub struct LoggerOptions {
    // Batching
    pub(crate) flush_at_bytes: usize,
    pub(crate) flush_at_messages: u16,
    pub(crate) flush_interval: Duration,

    // Buffer pool // ! TODO setters
    pub(crate) buffer_pool_size: usize,
    pub(crate) buffer_pool_capacity: usize,

    // Behavior
    pub(crate) context: Map<String, Value>,
    pub(crate) min_level: LogLevel,
    pub(crate) timestamp_format: Option<&'static str>,
    pub(crate) timestamp_key: &'static str,
    pub(crate) pretty: bool,
}

impl Default for LoggerOptions {
    fn default() -> Self {
        LoggerOptions {
            context: Map::new(),
            flush_at_bytes: 64 * 1024,
            flush_at_messages: 100,
            min_level: LogLevel::Debug,
            flush_interval: Duration::from_secs(1),
            timestamp_format: None,
            timestamp_key: "timestamp",
            pretty: false,
            buffer_pool_size: 64,
            buffer_pool_capacity: 1024,
        }
    }
}

impl LoggerOptions {
    // Sets a key, value pair that will be added to all of the logs that are produced
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

    /// How many bytes to buffer before flushing. Default is 64kb
    #[must_use = "call `.init()` to create a Logger"]
    pub fn flush_at_bytes(mut self, flush_at_bytes: usize) -> Self {
        if flush_at_bytes == 0 {
            eprintln!(
                "Provided 'flush_at_bytes' is invalid, using {}",
                self.flush_at_bytes
            )
        } else {
            self.flush_at_bytes = flush_at_bytes
        }
        self
    }

    /// How many messages to hold in memory before flushing. Default is 100
    #[must_use = "call `.init()` to create a Logger"]
    pub fn flush_at_messages(mut self, flush_at_messages: u16) -> Self {
        if flush_at_messages == 0 {
            eprintln!(
                "Provided 'flush_at_messages' is invalid, using {}",
                self.flush_at_messages
            )
        } else {
            self.flush_at_messages = flush_at_messages
        }
        self
    }

    /// How long to wait before flushing if either `flush_at_bytes` or `flush_at_messages` are not past their thresholds.
    /// Default is 1 second.
    #[must_use = "call `.init()` to create a Logger"]
    pub fn flush_interval(mut self, interval: Duration) -> Self {
        self.flush_interval = interval;
        self
    }

    /// Minimum log level to use. Anything below will not be logged.
    /// From left to right: Debug, Info, Warn, Error. Default is Debug.
    /// If you set the `min_level` to Warn, then Debug and Info WILL NOT show in your logs.
    #[must_use = "call `.init()` to create a Logger"]
    pub fn min_level(mut self, level: LogLevel) -> Self {
        self.min_level = level;
        self
    }

    /// Set a custom timestamp format
    /// Use these guides as reference:
    /// <https://docs.rs/chrono/latest/chrono/#formatting-and-parsing> &
    /// <https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers>
    #[must_use = "call `.init()` to create a Logger"]
    pub fn timestamp_format(mut self, timestamp_format: &'static str) -> Self {
        self.timestamp_format = Some(timestamp_format);

        self
    }

    /// Remap the timestamp key from `timestamp` to something else like `time`
    /// TODO!
    #[must_use = "call `.init()` to create a Logger"]
    pub fn timestamp_key(mut self, timestamp_key: &'static str) -> Self {
        self.timestamp_key = timestamp_key;
        self
    }

    /// Whether to use multi-line JSON log lines. Default `false`
    #[must_use = "call `.init()` to create a Logger"]
    pub fn pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }

    // Initializes the logger and returns it
    #[must_use = "Logger must be kept to write logs. For example: logger.info()"]
    pub fn init(self) -> Logger {
        assert!(
            LOGGER_INITIALIZED.swap(true, Ordering::SeqCst),
            "Logger already initialized! Only call .init() once"
        );

        let (sender, worker) = mpsc::channel::<Vec<u8>>();

        // Pre allocate a few buffers into the pool
        let buffer_pool = Arc::new(ArrayQueue::new(self.buffer_pool_size));
        for _ in 0..self.buffer_pool_size {
            let _ = buffer_pool.push(Vec::with_capacity(self.buffer_pool_capacity));
        }

        // Run in background
        let worker = Logger::handle_messages(
            worker,
            Arc::clone(&buffer_pool),
            self.flush_at_bytes,
            self.flush_at_messages,
            self.flush_interval,
        );

        Logger {
            min_level: self.min_level,
            buffer_pool,
            buffer_pool_capacity: self.buffer_pool_capacity,
            timestamp_format: self.timestamp_format,
            pretty: self.pretty,
            context: self.context,
            sender: Some(sender),
            worker: Some(worker),
        }
    }
}
