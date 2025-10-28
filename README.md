# sjl - Simple JSON Logger

 📦 **[crates.io](https://crates.io/crates/sjl)** | 📚 **[docs.rs](https://docs.rs/sjl)**

### Why?
I mostly need JSON logging without the quirks: [enums that serialize correctly](https://github.com/tokio-rs/tracing/issues/3051) and [clean output out of the box](https://josevalerio.com/rust-json-logging), *not* escaped strings.

I built this because the [tracing crate](https://crates.io/crates/tracing)'s `valuable` support has been behind an [unstable feature flag for over three years](https://github.com/tokio-rs/tracing/discussions/1906) and the  [slog](https://crates.io/crates/slog) crate also doesn't seem to provide this..

If you want a simple JSON logger, this might be useful for you too.




 ## Installation

 ```bash
 cargo add sjl
 ```

 ## Usage
 ```rust
use sjl::{debug, error, info, warn, LogLevel, Logger, RGB};
use serde::Serialize;
use serde_json::json;

#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
    verified: bool,
}

#[derive(Serialize)]
enum Status {
    Active,
    Inactive,
    RateLimited { retry_after: u32 },
}

#[derive(Serialize)]
struct Address {
    street: String,
    city: String,
    country: String,
}

#[derive(Serialize)]
struct OrderItem {
    sku: String,
    name: String,
    quantity: u32,
    price: f64,
    status: Status,
}

#[derive(Serialize)]
struct Order {
    order_id: String,
    user: User,
    status: Status,
    shipping_address: Address,
    items: Vec<OrderItem>,
    metadata: Metadata,
}

#[derive(Serialize)]
struct Metadata {
    tags: Vec<String>,
    priority: u8,
    notes: Option<String>,
}

fn main() {
    // Initialize once at startup
    Logger::init()
        .min_level(LogLevel::Debug)       // Minimum log level (default: Debug)
        .batch_size(100)                  // Logs per batch (default: 50)
        .batch_duration_ms(100)           // Max ms before flush (default: 50)
        .buffer_size(5000)                // Channel capacity (default: 1024)
        .timestamp_format("%Y-%m-%dT%H:%M:%S%.3fZ")  // ISO 8601 (default)
        .debug_color(RGB::new(38, 45, 56))   // Customize colors
        .info_color(RGB::new(15, 115, 255))
        .warn_color(RGB::new(247, 155, 35))
        .error_color(RGB::new(255, 0, 0))
        .build();

    // 1. Simple string messages
    debug!("Application started");

    // 2. String with message context
    info!("Server listening", "0.0.0.0:8080");

    // 3. Struct data (works seamlessly)
    info!(User {
        id: 1,
        name: "Alice".into(),
        verified: true
    });

    // 4. Message + struct data
    info!("User authenticated", User {
        id: 1,
        name: "Alice".into(),
        verified: true
    });

    // 5. Enum variants serialize correctly
    warn!(Status::Active);
    warn!(Status::RateLimited { retry_after: 60 });

    // 6. Ad-hoc JSON without defining structs
    error!(json!({
        "error": "connection_failed",
        "host": "db.example.com",
        "port": 5432
    }));

    // 7. Message + ad-hoc JSON
    error!("Database connection failed", json!({
        "host": "db.example.com",
        "port": 5432,
        "retry_count": 3
    }));

    // 8. Complex nested: Vec of structs containing enums
    info!(
        "Order processed",
        Order {
            order_id: "ORD-2024-001".into(),
            user: User {
                id: 42,
                name: "John Doe".into(),
                verified: true,
            },
            status: Status::Active,
            shipping_address: Address {
                street: "123 Main St".into(),
                city: "San Francisco".into(),
                country: "USA".into(),
            },
            items: vec![
                OrderItem {
                    sku: "WIDGET-001".into(),
                    name: "Premium Widget".into(),
                    quantity: 2,
                    price: 29.99,
                    status: Status::Active,
                },
                OrderItem {
                    sku: "GADGET-002".into(),
                    name: "Super Gadget".into(),
                    quantity: 1,
                    price: 49.99,
                    status: Status::RateLimited { retry_after: 30 },
                },
            ],
            metadata: Metadata {
                tags: vec!["express".into(), "gift".into()],
                priority: 1,
                notes: Some("Handle with care".into()),
            },
        }
    );
}
```

### Output
```json
{"level":"DEBUG","timestamp":"2025-10-28T10:14:41.229Z","data":"Application started"}
{"level":"INFO","timestamp":"2025-10-28T10:14:41.229Z", "message": "Server listening","data":"0.0.0.0:8080"}
{"level":"INFO","timestamp":"2025-10-28T10:14:41.229Z","data":{"id":1,"name":"Alice","verified":true}}
{"level":"INFO","timestamp":"2025-10-28T10:14:41.229Z", "message": "User authenticated","data":{"id":1,"name":"Alice","verified":true}}
{"level":"WARN","timestamp":"2025-10-28T10:14:41.229Z","data":"Active"}
{"level":"WARN","timestamp":"2025-10-28T10:14:41.229Z","data":{"RateLimited":{"retry_after":60}}}
{"level":"ERROR","timestamp":"2025-10-28T10:14:41.229Z","data":{"error":"connection_failed","host":"db.example.com","port":5432}}
{"level":"ERROR","timestamp":"2025-10-28T10:14:41.229Z", "message": "Database connection failed","data":{"host":"db.example.com","port":5432,"retry_count":3}}
{"level":"INFO","timestamp":"2025-10-28T10:14:41.229Z", "message": "Order processed","data":{"items":[{"name":"Premium Widget","price":29.99,"quantity":2,"sku":"WIDGET-001","status":"Active"},{"name":"Super Gadget","price":49.99,"quantity":1,"sku":"GADGET-002","status":{"RateLimited":{"retry_after":30}}}],"metadata":{"notes":"Handle with care","priority":1,"tags":["express","gift"]},"order_id":"ORD-2024-001","shipping_address":{"city":"San Francisco","country":"USA","street":"123 Main St"},"status":"Active","user":{"id":42,"name":"John Doe","verified":true}}}
```


