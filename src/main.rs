use std::{collections::HashMap, num::NonZeroUsize, thread, time::Duration};

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
    let logger = LoggerOptions::default()
        .flush_interval(Duration::from_secs(5))
        .max_messages(2)
        .init();

    logger.info("yeah", ());
    logger.info("yeah2", ());
    logger.info("yeah3", ());

    thread::sleep(Duration::from_secs(5));
    thread::sleep(Duration::from_secs(5));

    logger.info("yeah4", ());
    logger.info("yeah5", ());
}
