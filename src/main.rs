use std::{collections::HashMap, num::NonZeroUsize, thread, time::Duration};

use serde::Serialize;
use serde_json::Map;
use sjl::{Logger, json};

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

    let logger = Logger::builder().context("service", "shipments").init();

    logger.debug("Debug", ());
    logger.info("Info", ());
    logger.warn("warn", ());
    logger.error("error", ());
    logger.error(
        format!("tomato soup {}", 12),
        json!({"message": "from jose"}),
    );
}
