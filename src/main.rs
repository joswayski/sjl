use serde::Serialize;
use serde_json::json;
use sjl::{LogLevel, Logger, RGB, debug, error, info, warn};

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
        .min_level(LogLevel::Debug) // Minimum log level (default: Debug)
        .batch_size(100) // Logs per batch (default: 50)
        .batch_duration_ms(100) // Max ms before flush (default: 50)
        .buffer_size(5000) // Channel capacity (default: 1024)
        .timestamp_format("%Y-%m-%dT%H:%M:%S%.3fZ") // ISO 8601 (default)
        .debug_color(RGB::new(38, 45, 56)) // Customize colors
        .info_color(RGB::new(15, 115, 255))
        .warn_color(RGB::new(247, 155, 35))
        .error_color(RGB::new(255, 0, 0))
        // Call this at the end
        .build();

    // Strings
    debug!("App started");
    info!("Server listening", "0.0.0.0:8080");

    // Structs
    info!(User {
        id: 1,
        name: "Alice".into()
    });
    info!(
        "User authenticated",
        User {
            id: 1,
            name: "Alice".into()
        }
    );

    // Enums (serialize correctly!)
    warn!(Status::Active);
    warn!(Status::RateLimited { retry_after: 60 });

    // Ad-hoc JSON
    error!(json!({
        "error": "connection_failed",
        "host": "db.example.com"
    }));

    // Complex: Vec of structs with enums
    info!(
        "Order processed",
        Order {
            user: User {
                id: 42,
                name: "John".into()
            },
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
        }
    );
}
