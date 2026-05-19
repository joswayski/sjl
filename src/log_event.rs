use serde::Serialize;
use serde_json::{Map, Value};

use crate::timestamp::FormattedTimestamp;

#[derive(Serialize)]
pub(crate) struct LogEvent<'a, Data: Serialize> {
    pub(crate) timestamp: FormattedTimestamp,
    pub(crate) level: &'a str,
    pub(crate) message: &'a str,
    #[serde(flatten)]
    pub(crate) context: &'a Map<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) data: Option<&'a Data>,
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use serde_json::{Map, Value, json};
    #[derive(Serialize, Deserialize)]
    enum UserType {
        Basic,
        Admin { access: String },
    }

    #[derive(Serialize, Deserialize)]
    struct User {
        name: String,
        user_type: UserType,
    }

    use crate::{
        LogLevel,
        log_event::LogEvent,
        timestamp::{DEFAULT_TS_FORMAT, FormattedTimestamp},
    };

    #[test]
    fn test_serializes() {
        let user = User {
            name: "user1".to_string(),
            user_type: UserType::Admin {
                access: "full".to_string(),
            },
        };
        let mut test_map = Map::new();
        test_map.insert("test_map".to_string(), Value::String("test_value".into()));
        test_map.insert("user".to_string(), serde_json::to_value(user).unwrap());

        let ts = FormattedTimestamp::new(Some(DEFAULT_TS_FORMAT));
        let event = LogEvent {
            level: LogLevel::Info.as_str(),
            message: "Saul Goodman",
            timestamp: ts,
            data: Some(&json!({"sample_key": "sample_data"})),
            context: &test_map,
        };

        let result = serde_json::to_string(&event).unwrap();
        let parsed_result: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert!(result.contains("\"level\":\"info\""));
        assert!(result.contains("\"message\":\"Saul Goodman\""));
        assert!(result.contains("\"sample_key\":\"sample_data\""));
        assert!(result.contains("\"test_map\":\"test_value\""));
        assert!(parsed_result["data"].is_object());
        assert_eq!(parsed_result["data"]["sample_key"], "sample_data");
        assert!(parsed_result["timestamp"].as_str().unwrap().ends_with("Z"));

        println!("{}", parsed_result);
        assert_eq!(
            parsed_result["user"]["user_type"]["Admin"]["access"],
            "full"
        );
    }
}
