# sjl - Simple JSON Logger

 📦 **[crates.io](https://crates.io/crates/sjl)** | 📚 **[docs.rs](https://docs.rs/sjl)**

### Why?
The most popular logging crate, [tracing](https://crates.io/crates/tracing), has [problems with nested JSON](https://www.reddit.com/r/rust/comments/1k75jvc/how_can_i_emit_a_tracing_event_with_an_unescaped/) unless you use the `valuable` crate with it which is [unstable and behind a feature flag for 3 years]((https://github.com/tokio-rs/tracing/discussions/1906))... but that [still has issues with enums](https://github.com/tokio-rs/tracing/issues/3051) and doesn't feel natural to use with `.as_value()` everywhere.  The [slog](https://crates.io/crates/slog) crate has similar issues—I've written about both [here](https://josevalerio.com/rust-json-logging).

If you just want a simple JSON logger, you might find this useful.

## Features
- Batched, non-blocking writes
- Graceful shutdown (flushes on exit)
- Falls back to sync writes if buffer is full
- Customizable colors, timestamps, batch sizes
- Works with any `Serialize` type
- Macros! `debug!()`, `info!()`, `warn!()`, and `error!()`


 ## Installation

 ```bash
 cargo add sjl
 ```

## Usage
```rust
use sjl::{debug, error, info, warn, LogLevel, Logger, RGB};
use serde::Serialize;
use serde_json::json;

#[derive(Serialize)] // This is all you need
struct User {
    id: u64,
    name: String,
}

#[derive(Serialize)]
enum Status {
    Active,
    RateLimited { retry_after: u32 },
}

#[derive(Serialize)]
struct Order {
    user: User,
    items: Vec<OrderItem>,
}

#[derive(Serialize)]
struct OrderItem {
    name: String,
    price: f64,
    status: Status,
}

fn main() {
    // Initialize once at startup
    Logger::init()
        // Optional config
        .min_level(LogLevel::Debug)       // Minimum log level (default: Debug)
        .batch_size(100)                  // Logs per batch (default: 50)
        .batch_duration_ms(100)           // Max ms before flush (default: 50)
        .buffer_size(5000)                // Channel capacity (default: 1024)
        .timestamp_format("%Y-%m-%dT%H:%M:%S%.3fZ")  // ISO 8601 (default)
        .debug_color(RGB::new(38, 45, 56))   // Customize colors
        .info_color(RGB::new(15, 115, 255))
        .warn_color(RGB::new(247, 155, 35))
        .error_color(RGB::new(255, 0, 0))
        // Call this at the end
        .build(); 

    // Strings
    debug!("App started");
    info!("Server listening", "0.0.0.0:8080");

    // Structs
    info!(User { id: 1, name: "Alice".into() });
    info!("User authenticated", User { id: 1, name: "Alice".into() });

    // Enums (serialize correctly!)
    warn!(Status::Active);
    warn!(Status::RateLimited { retry_after: 60 });

    // Ad-hoc JSON
    error!(json!({
        "error": "connection_failed",
        "host": "db.example.com"
    }));

    // Complex: Vec of structs with enums
    info!("Order processed", Order {
        user: User { id: 42, name: "John".into() },
        items: vec![
            OrderItem {
                name: "Widget".into(),
                price: 29.99,
                status: Status::Active,
            },
            OrderItem {
                name: "Gadget".into(),
                price: 49.99,
                status: Status::RateLimited { retry_after: 30 },
            },
        ],
    });
}
```

### Output
```json
{"level":"DEBUG","timestamp":"2025-10-28T17:29:17.784Z","data":"Application started"}
{"level":"INFO","timestamp":"2025-10-28T17:29:17.784Z", "message": "Server listening","data":"0.0.0.0:8080"}
{"level":"INFO","timestamp":"2025-10-28T17:29:17.784Z","data":{"id":1,"name":"Alice","verified":true}}
{"level":"INFO","timestamp":"2025-10-28T17:29:17.784Z", "message": "User authenticated","data":{"id":1,"name":"Alice","verified":true}}
{"level":"WARN","timestamp":"2025-10-28T17:29:17.784Z","data":"Active"}
{"level":"WARN","timestamp":"2025-10-28T17:29:17.784Z","data":{"RateLimited":{"retry_after":60}}}
{"level":"ERROR","timestamp":"2025-10-28T17:29:17.784Z","data":{"error":"connection_failed","host":"db.example.com","port":5432}}
{"level":"ERROR","timestamp":"2025-10-28T17:29:17.784Z", "message": "Database connection failed","data":{"host":"db.example.com","port":5432,"retry_count":3}}
{"level":"INFO","timestamp":"2025-10-28T17:29:17.784Z", "message": "Order processed","data":{"items":[{"name":"Premium Widget","price":29.99,"quantity":2,"sku":"WIDGET-001","status":"Active"},{"name":"Super Gadget","price":49.99,"quantity":1,"sku":"GADGET-002","status":{"RateLimited":{"retry_after":30}}}],"metadata":{"notes":"Handle with care","priority":1,"tags":["express","gift"]},"order_id":"ORD-2024-001","shipping_address":{"city":"San Francisco","country":"USA","street":"123 Main St"},"status":"Active","user":{"id":42,"name":"John Doe","verified":true}}}
```



## Roadmap
Feel free to open an issue if you'd like to see something else!
- [ ] Structured context fields (e.g., add `service`, `env` to all logs)
