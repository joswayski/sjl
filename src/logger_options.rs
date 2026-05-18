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

const DEFAULT_FLUSH_AT_BYTES: usize = 64 * 2048;
const DEFAULT_FLUSH_AT_MESSAGES: u16 = 100;
const DEFAULT_BUFFER_POOL_SIZE: usize = 64;
const DEFAULT_BUFFER_POOL_INITIAL_CAPACITY: usize = 2048;
const DEFAULT_BUFFER_POOL_MAX_CAPACITY: usize = 20 * DEFAULT_BUFFER_POOL_INITIAL_CAPACITY;
const RESERVED_FIELD_NAMES: &[&str; 4] = &["timestamp", "level", "message", "data"];

#[must_use = "LoggerOptions does nothing until you call `.init()`"]
pub struct LoggerOptions {
    // Batching
    pub(crate) flush_at_bytes: usize,
    pub(crate) flush_at_messages: u16,
    pub(crate) flush_interval: Duration,

    // Buffer pool
    pub(crate) buffer_pool_size: usize,
    pub(crate) buffer_pool_initial_capacity: usize,
    pub(crate) buffer_pool_max_capacity: usize,

    // Behavior
    pub(crate) context: Map<String, Value>, // ! TODO add reserved field names again
    pub(crate) min_level: LogLevel,
    pub(crate) timestamp_format: Option<&'static str>,
    pub(crate) timestamp_key: &'static str, // TODO allow overriding
    pub(crate) pretty: bool,
}

impl Default for LoggerOptions {
    fn default() -> Self {
        LoggerOptions {
            context: Map::new(),
            flush_at_bytes: DEFAULT_FLUSH_AT_BYTES,
            flush_at_messages: DEFAULT_FLUSH_AT_MESSAGES,
            min_level: LogLevel::Debug,
            flush_interval: Duration::from_secs(1),
            timestamp_format: None,
            timestamp_key: "timestamp",
            pretty: false,
            buffer_pool_size: DEFAULT_BUFFER_POOL_SIZE,
            buffer_pool_initial_capacity: DEFAULT_BUFFER_POOL_INITIAL_CAPACITY,
            buffer_pool_max_capacity: DEFAULT_BUFFER_POOL_MAX_CAPACITY,
        }
    }
}

impl LoggerOptions {
    // Sets a key, value pair that will be added to all of the logs that are produced
    #[must_use = "call `.init()` to create a Logger"]
    pub fn context<V: Serialize>(mut self, key: impl Into<String>, value: V) -> Self {
        let key = key.into();
        assert!(
            !RESERVED_FIELD_NAMES.contains(&key.as_str()),
            "context key '{key}' is reserved. Reserved keys: {RESERVED_FIELD_NAMES:?}."
        );

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

    /// How many bytes to buffer before flushing. Default is `DEFAULT_FLUSH_AT_BYTES`
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

    /// How many messages to hold in memory before flushing. Default is `DEFAULT_FLUSH_AT_MESSAGES`
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

    /// How big the initial buffer pool should be to avoid new allocations per log
    /// This creates a buffer pool of Vec<u8> that are reused.
    /// Set this to your estimate of concurrent inflight logs for your application.
    /// Default is 64.
    #[must_use = "call `.init()` to create a Logger"]
    pub fn buffer_pool_size(mut self, buffer_pool_size: usize) -> Self {
        if buffer_pool_size == 0 {
            eprintln!(
                "Provided 'buffer_pool_size' is invalid, using {}",
                self.buffer_pool_size
            )
        } else {
            self.buffer_pool_size = buffer_pool_size;
        }
        self
    }

    /// How big each log is. This is used to preallocate buffers in a pool so they can be reused.
    /// There is also a `buffer_pool_max_capacity` which will trim Vec<u8>'s back down
    /// if they get resized over the limit.
    /// Set this to your estimate of how big your logs are
    /// Default is 2kb.
    #[must_use = "call `.init()` to create a Logger"]
    pub fn buffer_pool_initial_capacity(mut self, buffer_pool_initial_capacity: usize) -> Self {
        if buffer_pool_initial_capacity == 0 {
            eprintln!(
                "Provided 'buffer_pool_initial_capacity' is invalid, using {}",
                self.buffer_pool_initial_capacity
            )
        } else {
            self.buffer_pool_initial_capacity = buffer_pool_initial_capacity;
        }
        self
    }

    /// The absolute max size a buffer can grow to before being shrunk
    /// If you're hitting this often, it might be good to increase the `buffer_pool_initial_capacity`
    #[must_use = "call `.init()` to create a Logger"]
    pub fn buffer_pool_max_capacity(mut self, buffer_pool_max_capacity: usize) -> Self {
        if buffer_pool_max_capacity == 0 {
            eprintln!(
                "Provided 'buffer_pool_max_capacity' is invalid, using {}",
                self.buffer_pool_max_capacity
            )
        } else {
            self.buffer_pool_max_capacity = buffer_pool_max_capacity;
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

    // /// Remap the timestamp key from `timestamp` to something else like `time`
    // /// TODO!
    // #[must_use = "call `.init()` to create a Logger"]
    // pub fn timestamp_key(mut self, timestamp_key: &'static str) -> Self {
    //     self.timestamp_key = timestamp_key;
    //     self
    // }

    /// Whether to use multi-line JSON log lines. Default `false`
    #[must_use = "call `.init()` to create a Logger"]
    pub fn pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }

    // Initializes the logger and returns it
    #[must_use = "Logger must be kept to write logs. For example: logger.info()"]
    pub fn init(mut self) -> Logger {
        assert!(
            !LOGGER_INITIALIZED.swap(true, Ordering::SeqCst),
            "Logger already initialized! Only call .init() once"
        );

        if self.buffer_pool_initial_capacity >= self.buffer_pool_max_capacity {
            eprintln!(
                "buffer_pool_max_capacity ({}) < buffer_pool_initial_capacity ({}); clamping max to capacity",
                self.buffer_pool_max_capacity, self.buffer_pool_initial_capacity
            );

            self.buffer_pool_max_capacity = self.buffer_pool_initial_capacity;
        }

        let (sender, worker) = crossbeam_channel::unbounded::<Vec<u8>>();

        // Pre allocate a few buffers into the pool
        let buffer_pool = Arc::new(ArrayQueue::new(self.buffer_pool_size));
        for _ in 0..self.buffer_pool_size {
            let _ = buffer_pool.push(Vec::with_capacity(self.buffer_pool_initial_capacity));
        }

        // Run in background
        let worker = Logger::handle_messages(
            worker,
            Arc::clone(&buffer_pool),
            self.buffer_pool_max_capacity,
            self.buffer_pool_initial_capacity,
            self.flush_at_bytes,
            self.flush_at_messages,
            self.flush_interval,
        );

        Logger {
            min_level: self.min_level,
            buffer_pool,
            buffer_pool_initial_capacity: self.buffer_pool_initial_capacity,
            timestamp_format: self.timestamp_format,
            pretty: self.pretty,
            context: self.context,
            sender: Some(sender),
            worker: Some(worker),
        }
    }
}
