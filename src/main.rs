use std::{collections::HashMap, thread, time::Duration};

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

    thread::sleep(Duration::from_secs(5));
    logger.info("yeah2", ());
    logger.info("yeah3", ());
    thread::sleep(Duration::from_secs(5));

    logger.info("yeah4", ());
    logger.info("yeah5", ());
}
