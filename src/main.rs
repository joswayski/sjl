use sjl::{LogLevel, LoggerOptions};
use std::time::Duration;

fn main() {
    let logger = LoggerOptions::default()
        // Context are k/v pairs that are added to every log line
        // use these for identifiers like service, environment, version, etc.
        .context("service", "payments")
        .context("environment", "production")
        // Minimum severity that actually gets emitted.
        // For example, setting this to Info will not show Debug logs
        // Hierarchy: Debug < Info < Warn < Error
        .min_level(LogLevel::Warn)
        // Batching
        // Flush once the batch reaches this many bytes
        .flush_at_bytes(1_000)
        // ...or once we have this many messages
        .flush_at_messages(100)
        // ...or once this much time has passed since the last flush.
        // Whatever comes first wins.
        .flush_interval(Duration::from_millis(250))
        // Buffer pool
        // How many buffers to keep in the pool
        // Set this to around your expected concurrent in-flight log count
        .buffer_pool_size(20)
        // Starting capacity (in bytes) of each buffer. Tune this to your typical log size
        // so that hot path logging never has to grow the buffers
        .buffer_pool_initial_capacity(4_000)
        // Hard cap on how big the buffers can get. Any that exceed this size
        // will get shrunk back down before being returned to the pool
        // So that one giant log can't be a memory hog.
        // Oversized logs also trigger occasional warnings
        .buffer_pool_max_capacity(100_000)
        // Rename the `timestamp` field in the output
        .timestamp_key("time")
        // Custom chrono strftime format. Default is RFC 3339 with milliseconds.
        // Build your own from here: https://docs.rs/chrono/latest/chrono/format/strftime/index.html
        .timestamp_format("%FT%I:%M:%S%p")
        // Pretty-print JSON using multiple lines. Default is compact, single line.
        .pretty(true)
        // Spawns a background worker thread and returns the logger. Only call this once or it'll panic.
        .init();

    logger.error("Saul Goodman!", ());
}
