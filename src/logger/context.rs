use hashbrown::HashMap;
use serde_json::Value;

pub type LoggerContext = HashMap<String, Value>;
