use serde::Serialize;
use serde_json::{Map, Value};

use crate::timestamp::FormattedTimestamp;

#[derive(Serialize)]
pub(crate) struct LogEvent<'a, Data: Serialize> {
    pub(crate) timestamp: FormattedTimestamp<'a>,
    pub(crate) level: &'a str,
    pub(crate) message: &'a str,
    #[serde(flatten)]
    pub(crate) context: &'a Map<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) data: Option<&'a Data>,
}
