use chrono::{DateTime, Utc};
use serde::{Serialize, Serializer};

pub const DEFAULT_TS_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.3fZ";
pub(crate) struct FormattedTimestamp<'a> {
    pub(crate) dt: DateTime<Utc>,
    pub(crate) fmt: &'a str,
}

impl Serialize for FormattedTimestamp<'_> {
    // This a voids an allocation when creating the timestamp
    // before sending it through the channel. It streams chrono's Display output
    // through serde into the output buffer/
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.dt.format(self.fmt))
    }
}

impl FormattedTimestamp<'_> {
    pub(crate) fn new(tz_format: Option<&'static str>) -> Self {
        FormattedTimestamp {
            dt: Utc::now(),
            fmt: tz_format.unwrap_or(DEFAULT_TS_FORMAT),
        }
    }
}
