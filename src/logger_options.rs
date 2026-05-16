use serde::Serialize;
use serde_json::{Map, Value};

use crate::Logger;

#[must_use = "LoggerOptions does nothing until you call `.init()`"]
pub struct LoggerOptions {
    pub(crate) context: Map<String, Value>,
}
impl Default for LoggerOptions {
    fn default() -> Self {
        LoggerOptions {
            context: Map::new(),
        }
    }
}

impl LoggerOptions {
    #[must_use = "call `.init()` to create a Logger"]
    pub fn context<V: Serialize>(mut self, key: impl Into<String>, value: V) -> Self {
        let key = key.into();
        match serde_json::to_value(value) {
            // If it's serializable to an object, all is good
            Ok(value) => {
                self.context.insert(key, value);
            }
            // If we failed to parse, return an error
            Err(serialize_error) => {
                eprintln!(
                    "Error serializing context value for key '{}'. It will not be included. \nError: {}",
                    key, serialize_error
                );
            }
        };

        self
    }

    #[must_use = "Logger must be kept to write logs. For example: logger.info()"]
    pub fn init(self) -> Logger {
        Logger {
            context: self.context,
        }
    }
}
