use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use crossbeam_queue::ArrayQueue;
use serde::Serialize;
use serde_json::{Map, Value, map::Entry};

use crate::{Logger, log_level::LogLevel};

pub static LOGGER_INITIALIZED: AtomicBool = AtomicBool::new(false);

const DEFAULT_FLUSH_AT_BYTES: usize = 64 * 2048;
const DEFAULT_FLUSH_AT_MESSAGES: usize = 100;
const DEFAULT_FLUSH_INTERVAL: Duration = Duration::from_secs(1);
const DEFAULT_BUFFER_POOL_SIZE: usize = 10;
const DEFAULT_BUFFER_POOL_INITIAL_CAPACITY: usize = 2048;
const DEFAULT_BUFFER_POOL_MAX_CAPACITY: usize = 20 * DEFAULT_BUFFER_POOL_INITIAL_CAPACITY;
const RESERVED_FIELD_NAMES: &[&str; 3] = &["level", "message", "data"];

#[must_use = "LoggerOptions does nothing until you call `.init()`"]
pub struct LoggerOptions {
    // Batching
    pub(crate) flush_at_bytes: usize,
    pub(crate) flush_at_messages: usize,
    pub(crate) flush_interval: Duration,

    // Buffer pool
    pub(crate) buffer_pool_size: usize,
    pub(crate) buffer_pool_initial_capacity: usize,
    pub(crate) buffer_pool_max_capacity: usize,

    // Behavior
    pub(crate) context: Map<String, Value>,
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
            flush_interval: DEFAULT_FLUSH_INTERVAL,
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
    /// Sets a key, value pair that will be added to all of the logs that are produced
    /// Keys must be non-empty and not in the reserved set of (`level`, `message`, `data`)
    #[must_use = "call `.init()` to create a Logger"]
    pub fn context<V: Serialize>(mut self, key: impl Into<String>, value: V) -> Self {
        let key = key.into();
        assert!(!key.trim().is_empty(), "context key '{key}' is empty.");
        assert!(
            !RESERVED_FIELD_NAMES.contains(&key.as_str()),
            "context key '{key}' is reserved. Reserved keys: {RESERVED_FIELD_NAMES:?}."
        );

        match serde_json::to_value(value) {
            // If it's serializable to an object, all is good
            Ok(new_value) => match self.context.entry(key) {
                Entry::Occupied(mut entry) => {
                    eprintln!(
                        "SJL_WARN: You have a duplicate key '{}' being set in .context() calls. '{}' was overridden with '{}'",
                        entry.key(),
                        entry.get(),
                        new_value
                    );

                    entry.insert(new_value);
                }
                Entry::Vacant(entry) => {
                    entry.insert(new_value);
                }
            },
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

    /// How many bytes to buffer before flushing. Default is 128 KiB
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
    pub fn flush_at_messages(mut self, flush_at_messages: usize) -> Self {
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
    /// Default is 10.
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

    /// Initial byte capacity for each buffer in the pool.
    /// Set this to your estimate of how big your logs are so that the common case avoids reallocations.
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
    /// If you're hitting this often, it might be good to increase the `buffer_pool_initial_capacity`.
    /// Default is 40 KiB
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
    /// Default is RFC 3339 with millisecond precision (2024-01-15T14:30:00.123Z)
    #[must_use = "call `.init()` to create a Logger"]
    pub fn timestamp_format(mut self, timestamp_format: &'static str) -> Self {
        self.timestamp_format = Some(timestamp_format);

        self
    }

    /// Remap the timestamp key from `timestamp` to something else like `time`
    /// Default is `timestamp`
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

    fn validate(&mut self) {
        assert!(
            self.buffer_pool_initial_capacity <= self.buffer_pool_max_capacity,
            "buffer_pool_initial_capacity '{}' must be <= buffer_pool_max_capacity '{}'",
            self.buffer_pool_initial_capacity,
            self.buffer_pool_max_capacity
        );

        assert!(
            !self.context.contains_key(self.timestamp_key),
            "timestamp_key '{}' collides with a context key. Context keys show up at the top level with the timestamp, consider changing one of them",
            self.timestamp_key
        )
    }
    // Initializes the logger and returns it
    #[must_use = "Logger must be kept to write logs. For example: logger.info()"]
    pub fn init(mut self) -> Logger {
        assert!(
            // this is outside of validate so testing is easier since the logger gets dropped
            // when it goes outof scope
            !LOGGER_INITIALIZED.swap(true, Ordering::SeqCst),
            "Logger already initialized! Only call .init() once"
        );

        self.validate();

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
            timestamp_key: self.timestamp_key,
            pretty: self.pretty,
            context: self.context,
            sender: Some(sender),
            worker: Some(worker),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sets_defaults() {
        let log_opts = LoggerOptions::default();
        assert_eq!(log_opts.pretty, false);
        assert_eq!(log_opts.min_level, LogLevel::Debug);
        assert_eq!(log_opts.timestamp_key, "timestamp");
        assert_eq!(log_opts.timestamp_format, None); // sets none

        assert_eq!(log_opts.flush_interval, Duration::from_secs(1));
        assert_eq!(log_opts.flush_at_bytes, 64 * 2048);
        assert_eq!(log_opts.flush_at_messages, 100);

        assert_eq!(log_opts.context.keys().len(), 0);
        assert_eq!(log_opts.buffer_pool_size, 10);
        assert_eq!(log_opts.buffer_pool_initial_capacity, 2 * 1024);
        assert_eq!(log_opts.buffer_pool_max_capacity, 40 * 1024);
    }

    #[test]
    fn test_can_override_values() {
        let log_opts = LoggerOptions::default()
            .pretty(true)
            .min_level(LogLevel::Error)
            .timestamp_format("%Y-%m")
            .timestamp_key("poop")
            .flush_interval(Duration::from_secs(69420))
            .flush_at_bytes(69420)
            .flush_at_messages(69)
            .context("69", "420")
            .buffer_pool_size(69420)
            .buffer_pool_initial_capacity(69420)
            .buffer_pool_max_capacity(69420);

        assert_eq!(log_opts.pretty, true);
        assert_eq!(log_opts.min_level, LogLevel::Error);
        assert_eq!(log_opts.timestamp_key, "poop");
        assert_eq!(log_opts.timestamp_format, Some("%Y-%m")); // sets none

        assert_eq!(log_opts.flush_interval, Duration::from_secs(69420));
        assert_eq!(log_opts.flush_at_bytes, 69420);
        assert_eq!(log_opts.flush_at_messages, 69);

        assert_eq!(log_opts.context.keys().len(), 1);
        assert_eq!(log_opts.buffer_pool_size, 69420);
        assert_eq!(log_opts.buffer_pool_initial_capacity, 69420);
        assert_eq!(log_opts.buffer_pool_max_capacity, 69420);
    }

    #[test]
    fn test_init_happy_path() {
        let logger = LoggerOptions::default().init();

        assert_eq!(logger.pretty, false);
        assert_eq!(logger.min_level, LogLevel::Debug);
        assert_eq!(logger.timestamp_key, "timestamp");
        assert_eq!(logger.timestamp_format, None); // sets none
    }

    // cargo test -- --test-threads=1
    #[test]
    #[should_panic(expected = "Logger already initialized")]
    fn test_cant_initialize_more_than_one() {
        let _first = LoggerOptions::default().init();
        let _second = LoggerOptions::default().init();
    }

    #[test]
    #[should_panic(expected = "must be <=")]
    fn test_buffer_pool_initial_capacity_less_than_buffer_pool_max_capacity() {
        let mut opts = LoggerOptions::default()
            .buffer_pool_initial_capacity(100)
            .buffer_pool_max_capacity(20);

        opts.validate();
    }

    #[test]
    fn test_buffer_pool_sizes_are_valid() {
        let mut opts = LoggerOptions::default()
            .buffer_pool_initial_capacity(20)
            .buffer_pool_max_capacity(100);

        opts.validate();
    }

    #[test]
    #[should_panic(expected = "is reserved. Reserved keys")]
    fn test_setting_context_to_a_reserved_key() {
        let mut opts = LoggerOptions::default().context("timestamp", "poop");

        opts.validate();
    }

    #[test]
    #[should_panic(
        expected = "collides with a context key. Context keys show up at the top level with the timestamp, consider changing one of them"
    )]
    fn test_timestmap_key_collision_with_context() {
        let mut opts = LoggerOptions::default()
            .context("custom_timestamp", "poop")
            .timestamp_key("custom_timestamp");

        opts.validate();
    }

    #[test]
    #[should_panic(expected = " is empty.")]
    fn test_no_empty_context_keys_after_normalization() {
        let _ = LoggerOptions::default().context("     ", true);
    }

    #[test]
    fn test_overrides_duplicate_context_keys() {
        let ops = LoggerOptions::default()
            .context("name", "Jose")
            .context("name", "Valerio");

        assert_eq!(ops.context.keys().len(), 1);
        assert_eq!(
            ops.context.get("name").and_then(|v| v.as_str()),
            Some("Valerio")
        );
    }

    #[test]
    fn test_uses_default_flush_at_bytes_if_0() {
        let ops = LoggerOptions::default().flush_at_bytes(0);

        assert_eq!(ops.flush_at_bytes, 64 * 2048);
    }

    #[test]
    fn test_uses_default_flush_at_messages_if_0() {
        let ops = LoggerOptions::default().flush_at_messages(0);
        assert_eq!(ops.flush_at_messages, 100);
    }

    #[test]
    fn test_uses_default_buffer_pool_size_if_0() {
        let ops = LoggerOptions::default().buffer_pool_size(0);
        assert_eq!(ops.buffer_pool_size, 10);
    }

    #[test]
    fn test_uses_default_buffer_pool_initial_capacity_if_0() {
        let ops = LoggerOptions::default().buffer_pool_initial_capacity(0);
        assert_eq!(ops.buffer_pool_initial_capacity, 2048);
    }

    #[test]
    fn test_uses_default_buffer_pool_max_capacity_if_0() {
        let ops = LoggerOptions::default().buffer_pool_max_capacity(0);
        assert_eq!(ops.buffer_pool_max_capacity, 20 * 2048);
    }
}
