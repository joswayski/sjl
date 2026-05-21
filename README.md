# sjl - Simple JSON Logger

📦 **[crates.io](https://crates.io/crates/sjl)** | 📚 **[docs.rs](https://docs.rs/sjl)**

## What
It's a Simple JSON Logger. It logs JSON to `stderr`.


## Why?
The most popular logging crate, [tracing](https://crates.io/crates/tracing), has [problems with nested JSON](https://www.reddit.com/r/rust/comments/1k75jvc/how_can_i_emit_a_tracing_event_with_an_unescaped/) unless you use the `valuable` crate with it which is [unstable and behind a feature flag for 3 years](https://github.com/tokio-rs/tracing/discussions/1906)... but that [still has issues with enums](https://github.com/tokio-rs/tracing/issues/3051) and doesn't feel natural to use with `.as_value()` everywhere.  The [slog](https://crates.io/crates/slog) crate has similar issues—I've written about both [here](https://josevalerio.com/rust-json-logging).

If you just want a Simple JSON Logger, you might find this useful.



 ## Installation

 ```bash
 cargo add sjl
 ```

## Usage
```rust
use sjl::Logger;

fn main() {
    let logger = Logger::new();

    logger.info("Hello", ()); // 2nd param is optional data
}
```

### Outputs
```json
{"timestamp":"2026-05-21T02:45:03.456Z","level":"info","message":"Hello"}
```

## Extended Usage / Raison d'être
```rust
use serde::Serialize;
use sjl::LoggerOptions;

#[derive(Serialize)] // <-- All you need!
struct User {
    name: String,
    cars: Vec<Car>,
}

#[derive(Serialize)]
struct Car {
    make: String,
    model: String,
    transmission: Transmission,
}

#[derive(Serialize)]
enum Transmission {
    Automatic,
    Manual,
}

fn main() {
    let logger = LoggerOptions::default().pretty(true).init();

    let user = User {
        name: "Jose".into(),
        cars: vec![
            Car {
                make: "Toyota".into(),
                model: "Rav4".into(),
                transmission: Transmission::Manual,
            },
            Car {
                make: "Tesla".into(),
                model: "Cybertruck".into(),
                transmission: Transmission::Automatic,
            },
        ],
    };

    logger.info("Saul Goodman!", &user);
}
```

### Outputs
```json
{
  "timestamp": "2026-05-21T03:39:36.780Z",
  "level": "info",
  "message": "Saul Goodman!",
  "data": {
    "name": "Jose",
    "cars": [
    // No escaped strings!
      {
        "make": "Toyota",
        "model": "Rav4",
        "transmission": "Manual" // Enums render normally
      },
      {
        "make": "Tesla",
        "model": "Cybertruck",
        "transmission": "Automatic"
      }
    ]
  }
}
```

## All Options
```rust
use std::time::Duration;
use sjl::{LoggerOptions, LogLevel};

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
        // Spawns a background worker thread and returns the logger
        .init();

    logger.error("Saul Goodman!", ());

}
```

### Outputs
```json
{
  "time": "2026-05-21T03:35:04AM",
  "level": "error",
  "message": "Saul Goodman!",
  "environment": "production",
  "service": "payments"
}
```




## Running Tests
```bash
cargo llvm-cov --html
```