# sjl - Simple JSON Logger

 📦 **[crates.io](https://crates.io/crates/sjl)** | 📚 **[docs.rs](https://docs.rs/sjl)**

### Why?
The most popular logging crate, [tracing](https://crates.io/crates/tracing), has [problems with nested JSON](https://www.reddit.com/r/rust/comments/1k75jvc/how_can_i_emit_a_tracing_event_with_an_unescaped/) unless you use the `valuable` crate with it which is [unstable and behind a feature flag for 3 years](https://github.com/tokio-rs/tracing/discussions/1906)... but that [still has issues with enums](https://github.com/tokio-rs/tracing/issues/3051) and doesn't feel natural to use with `.as_value()` everywhere.  The [slog](https://crates.io/crates/slog) crate has similar issues—I've written about both [here](https://josevalerio.com/rust-json-logging).

If you just want a simple JSON logger, you might find this useful.

## Features
- Batched, non-blocking writes
- Graceful shutdown (flushes on exit)
- Falls back to sync writes if buffer is full
- Customizable colors, timestamps, batch sizes
- Pretty-printing mode for development
- Works with any `Serialize` type
- Macros! `debug!()`, `info!()`, `warn!()`, and `error!()`
- Global context fields that appear in every log message


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
enum OrderStatus {
    Pending,
    Shipped { tracking_number: String },
    Delivered,
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
    quantity: u32,
    status: OrderStatus,
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
        .pretty(true)                     // Pretty-print JSON (default: false)
        .debug_color(RGB::new(38, 45, 56))   // Customize colors
        .info_color(RGB::new(15, 115, 255))
        .warn_color(RGB::new(247, 155, 35))
        .error_color(RGB::new(255, 0, 0))
        // Context fields appear in EVERY log message at the top level
        .context("environment", "production")
        .context("service", "order-api")
        .context("metadata", json!({
            "instance_id": "i-1234567890abcdef0",
            "pod_name": "order-api-7d4f8c9b5-x8k2p",
            "git_sha": "abc123f"
        }))
        // Call this at the end
        .build(); 

    // Strings
    debug!("App started");
    info!("Server listening", "0.0.0.0:8080");

    // Structs
    info!(User { id: 1, name: "Alice".into() });
    info!("User authenticated", User { id: 1, name: "Alice".into() });

    // Enums (serialize correctly!)
    warn!(OrderStatus::Pending);
    warn!(OrderStatus::Shipped { tracking_number: "1Z999AA10123456784".into() });

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
                quantity: 2,
                status: OrderStatus::Shipped { tracking_number: "1Z999AA10123456784".into() },
            },
            OrderItem {
                name: "Gadget".into(),
                price: 49.99,
                quantity: 1,
                status: OrderStatus::Pending,
            },
        ],
    });
}
```

### Pretty Mode

When `.pretty(true)` is enabled, logs are formatted with indentation and newlines for easier reading during development:

```json
{
  "data": {
    "items": [
      {
        "name": "Widget",
        "price": 29.99,
        "quantity": 2,
        "status": {
          "Shipped": {
            "tracking_number": "1Z999AA10123456784"
          }
        }
      },
      {
        "name": "Gadget",
        "price": 49.99,
        "quantity": 1,
        "status": "Pending"
      }
    ],
    "user": {
      "id": 42,
      "name": "John"
    }
  },
  "environment": "production",
  "level": "INFO",
  "message": "Order processed",
  "metadata": {
    "git_sha": "abc123f",
    "instance_id": "i-1234567890abcdef0",
    "pod_name": "order-api-7d4f8c9b5-x8k2p"
  },
  "service": "order-api",
  "timestamp": "2025-10-31T01:54:41.972Z"
}
```



### Compact Mode (Default)

With `.pretty(false)` or omitted (default), logs are output as single-line JSON:

```json
{"level":"DEBUG","timestamp":"2025-10-31T01:57:41.170Z","data":"App started","environment": "production", "service": "order-api", "metadata": {"git_sha":"abc123f","instance_id":"i-1234567890abcdef0","pod_name":"order-api-7d4f8c9b5-x8k2p"}}
{"level":"INFO","timestamp":"2025-10-31T01:57:41.170Z","message":"Server listening","data":"0.0.0.0:8080","environment": "production", "service": "order-api", "metadata": {"git_sha":"abc123f","instance_id":"i-1234567890abcdef0","pod_name":"order-api-7d4f8c9b5-x8k2p"}}
{"level":"INFO","timestamp":"2025-10-31T01:57:41.170Z","data":{"id":1,"name":"Alice"},"environment": "production", "service": "order-api", "metadata": {"git_sha":"abc123f","instance_id":"i-1234567890abcdef0","pod_name":"order-api-7d4f8c9b5-x8k2p"}}
{"level":"INFO","timestamp":"2025-10-31T01:57:41.170Z","message":"User authenticated","data":{"id":1,"name":"Alice"},"environment": "production", "service": "order-api", "metadata": {"git_sha":"abc123f","instance_id":"i-1234567890abcdef0","pod_name":"order-api-7d4f8c9b5-x8k2p"}}
{"level":"WARN","timestamp":"2025-10-31T01:57:41.170Z","data":"Pending","environment": "production", "service": "order-api", "metadata": {"git_sha":"abc123f","instance_id":"i-1234567890abcdef0","pod_name":"order-api-7d4f8c9b5-x8k2p"}}
{"level":"WARN","timestamp":"2025-10-31T01:57:41.170Z","data":{"Shipped":{"tracking_number":"1Z999AA10123456784"}},"environment": "production", "service": "order-api", "metadata": {"git_sha":"abc123f","instance_id":"i-1234567890abcdef0","pod_name":"order-api-7d4f8c9b5-x8k2p"}}
{"level":"ERROR","timestamp":"2025-10-31T01:57:41.170Z","data":{"error":"connection_failed","host":"db.example.com"},"environment": "production", "service": "order-api", "metadata": {"git_sha":"abc123f","instance_id":"i-1234567890abcdef0","pod_name":"order-api-7d4f8c9b5-x8k2p"}}
{"level":"INFO","timestamp":"2025-10-31T01:57:41.170Z","message":"Order processed","data":{"items":[{"name":"Widget","price":29.99,"quantity":2,"status":{"Shipped":{"tracking_number":"1Z999AA10123456784"}}},{"name":"Gadget","price":49.99,"quantity":1,"status":"Pending"}],"user":{"id":42,"name":"John"}},"environment": "production", "service": "order-api", "metadata": {"git_sha":"abc123f","instance_id":"i-1234567890abcdef0","pod_name":"order-api-7d4f8c9b5-x8k2p"}}
```
