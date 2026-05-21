use serde::Serialize;
use sjl::{LoggerOptions, json};

#[derive(Serialize)] // <-- All you need!
struct User {
    name: String,
    cars: Vec<Car>,
}

#[derive(Serialize)]
struct Car {
    make: String,
    model: String,
    transmission: Transmission,
}

#[derive(Serialize)]
enum Transmission {
    Automatic,
    Manual,
}

fn main() {
    let logger = LoggerOptions::default().pretty(true).init();

    let user = User {
        name: "Jose".into(),
        cars: vec![
            Car {
                make: "Toyota".into(),
                model: "Rav4".into(),
                transmission: Transmission::Manual,
            },
            Car {
                make: "Tesla".into(),
                model: "Cybertruck".into(),
                transmission: Transmission::Automatic,
            },
        ],
    };

    logger.info("Saul Goodman!", &user);
}
