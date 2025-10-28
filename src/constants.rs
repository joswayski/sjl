/// Default size of the channel - max messages you can send at one time
pub const DEFAULT_BUFFER_SIZE: usize = 1024;

/// Default number of log messages to batch before flushing
pub const DEFAULT_BATCH_SIZE: usize = 50;

/// Default duration in milliseconds to wait before flushing a batch
pub const DEFAULT_BATCH_DURATION_MS: u64 = 50;

/// Default format for timestamps: 2025-10-23T15:30:45.123Z
pub const DEFAULT_TIMESTAMP_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.3fZ";
