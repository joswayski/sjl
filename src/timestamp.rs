use chrono::{DateTime, Utc};
use serde::{Serialize, Serializer};

pub const DEFAULT_TS_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.3fZ";
pub(crate) struct FormattedTimestamp {
    pub(crate) dt: DateTime<Utc>,
    pub(crate) fmt: &'static str,
}

impl Serialize for FormattedTimestamp {
    // This avoids an allocation when creating the timestamp
    // before sending it through the channel. It streams chrono's Display output
    // through serde into the output buffer/
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.dt.format(self.fmt))
    }
}

impl FormattedTimestamp {
    pub(crate) fn new(tz_format: Option<&'static str>) -> Self {
        FormattedTimestamp {
            dt: Utc::now(),
            fmt: tz_format.unwrap_or(DEFAULT_TS_FORMAT),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uses_default_timestamp_format() {
        let ts: FormattedTimestamp = FormattedTimestamp::new(None);
        assert_eq!(ts.fmt, DEFAULT_TS_FORMAT)
    }

    #[test]
    fn test_returns_timestamp() {
        let now = Utc::now();
        let ts: FormattedTimestamp = FormattedTimestamp::new(None);
        assert!(now.le(&ts.dt));
    }

    #[test]
    fn test_respects_time_format() {
        // https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers
        let format_output = "%Y-%b-%d-%a-%I-%p";

        let now = FormattedTimestamp::new(Some(format_output));
        let expected = now.dt.format(format_output).to_string();
        let serialized = serde_json::to_string(&now)
            .unwrap()
            .trim_matches('"')
            .to_string();

        assert_eq!(expected, serialized);
        assert_ne!(now.fmt, DEFAULT_TS_FORMAT);
        assert_ne!(now.dt.format(DEFAULT_TS_FORMAT).to_string(), expected);
    }
}
