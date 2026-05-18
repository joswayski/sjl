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

    let logger = Logger::builder()
        .context("beans", "shipments")
        .context("beans", "poop")
        .init();

    logger.debug("Debug", ());
    logger.info("Info", ());
    logger.warn("warn", ());
    logger.error("erro this is a really long message can you check it?this is a really long message can you check it?this is a really long message can you check it?this is a really long message can you check it?this is a really long message can you check it?this is a really long message can you check it?this is a really long message can you check it?this is a really long message can you check it?", ());
}
