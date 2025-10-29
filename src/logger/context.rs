use hashbrown::HashMap;
use serde_json::Value;

pub(crate) type LoggerContext = HashMap<String, Value>;
