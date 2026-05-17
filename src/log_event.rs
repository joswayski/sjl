use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Serialize)]
pub(crate) struct LogEvent<'a, Data: Serialize> {
    #[serde(flatten)]
    pub(crate) context: &'a Map<String, Value>,
    pub(crate) level: &'a str,
    pub(crate) timestamp: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) data: Option<&'a Data>,
    pub(crate) message: &'a str,
}
