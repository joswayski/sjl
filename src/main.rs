use std::{thread, time::Duration};

use sjl::{Logger, json};

fn main() {
    let logger = Logger::builder()
        .context("beans", "shipments")
        .context("beans", "poop") // demonstrates the duplicate-key warning at builder time
        .timestamp_key("poop")
        .init();

    // smoke test — confirms each level emits before we start the loop
    logger.debug("Debug", ());
    logger.info("Info", ());
    logger.warn("warn", ());
    logger.error("error", ());

    // ~1.5 KiB once serialized — used for the normal-traffic phases
    let payload = json!({
        "user_id": "user_1234567890abcdef",
        "session_id": "sess_abcdef0123456789abcdef0123456789",
        "request": {
            "method": "POST",
            "path": "/api/v1/orders/checkout",
            "headers": {
                "content-type": "application/json",
                "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
                "accept-encoding": "gzip, deflate, br",
                "x-request-id": "req_0123456789abcdef0123456789abcdef",
                "authorization": "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.payload.signature",
            },
            "body_preview": "x".repeat(600),
        },
        "metadata": {
            "ip": "192.168.1.100",
            "country": "US",
            "region": "CA",
            "device_fingerprint": "fp_".to_string() + &"a".repeat(64),
            "tags": ["checkout", "high-value", "returning-customer", "promo-applied"],
        },
    });

    // ~45 KiB — trips the oversized warning in handle_messages
    let huge = "x".repeat(45_000);

    let mut iter: u64 = 0;
    loop {
        iter += 1;

        // PHASE 1 — burst: trips flush_at_bytes (~85 logs) and flush_at_messages (100)
        eprintln!("\n[driver] iter {iter} :: phase 1 — bursting 250 logs");
        for i in 0..250 {
            logger.info(format!("iter {iter} burst {i}"), &payload);
        }

        // PHASE 2 — trickle: each gap > flush_interval (1s) so the timer path fires
        eprintln!("[driver] iter {iter} :: phase 2 — 5 trickle logs, 1.2s apart");
        for i in 0..5 {
            logger.info(format!("iter {iter} trickle {i}"), &payload);
            thread::sleep(Duration::from_millis(1200));
        }

        // PHASE 3 — idle: confirm the worker thread keeps spinning quietly
        eprintln!("[driver] iter {iter} :: phase 3 — idle 2s");
        thread::sleep(Duration::from_secs(2));

        // PHASE 4 — oversized: each log > buffer_pool_max_capacity (40 KiB)
        // first one always warns; subsequent ones throttle per the warn_every_n math
        eprintln!("[driver] iter {iter} :: phase 4 — 3 oversized (~45 KiB) logs");
        for i in 0..3 {
            logger.warn(format!("oversized {i}"), &huge);
        }
    }
}
