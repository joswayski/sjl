use serde::Serialize;
use serde_json::json;
use sjl::{LogLevel, Logger, RGB, debug, error, info, warn};

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
        .min_level(LogLevel::Debug) // Minimum log level (default: Debug)
        .batch_size(100) // Logs per batch (default: 50)
        .batch_duration_ms(100) // Max ms before flush (default: 50)
        .buffer_size(5000) // Channel capacity (default: 1024)
        .timestamp_format("%Y-%m-%dT%H:%M:%S%.3fZ") // ISO 8601 (default)
        .debug_color(RGB::new(38, 45, 56)) // Customize colors
        .info_color(RGB::new(15, 115, 255))
        .warn_color(RGB::new(247, 155, 35))
        .error_color(RGB::new(255, 0, 0))
        // Context fields appear in EVERY log message at the top level
        .context("environment", "production")
        .context("service", "order-api")
        .context(
            "metadata",
            json!({
                "instance_id": "i-1234567890abcdef0",
                "pod_name": "order-api-7d4f8c9b5-x8k2p",
                "git_sha": "abc123f"
            }),
        )
        .pretty(true) // Disabled for testing compact mode
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
    warn!(OrderStatus::Pending);
    warn!(OrderStatus::Shipped {
        tracking_number: "1Z999AA10123456784".into()
    });

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
                    quantity: 2,
                    status: OrderStatus::Shipped {
                        tracking_number: "1Z999AA10123456784".into()
                    },
                },
                OrderItem {
                    name: "Gadget".into(),
                    price: 49.99,
                    quantity: 1,
                    status: OrderStatus::Pending,
                },
            ],
        }
    );
}
