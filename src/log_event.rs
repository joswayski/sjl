use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Serialize)]
pub(crate) struct LogEvent<'a, Data> {
    #[serde(flatten)]
    pub(crate) context: &'a Map<String, Value>,
    pub(crate) level: &'a str,
    pub(crate) timestamp: &'a str,
    pub(crate) data: Option<&'a Data>,
    pub(crate) message: &'a str,
}
