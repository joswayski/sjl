use std::collections::HashMap;

use serde::Serialize;
use serde_json::Map;
use sjl::{Logger, LoggerOptions, json};

#[derive(Serialize)]
struct Blob {
    age: usize,
}

#[derive(Serialize)]
enum Status {
    Paid { amount: String, blob: Blob },
}
fn main() -> () {
    let logger = Logger::default();
    // or..
    let mut map = HashMap::new();
    map.insert(123, "bong");

    let logger = LoggerOptions::default()
        .context(
            "beans",
            Status::Paid {
                amount: 12.to_string(),
                blob: Blob { age: 15 },
            },
        )
        .init();

    logger.info("yeah", ());
}
