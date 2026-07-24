// src/rtds/message.rs

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RtdsMessage {
    pub topic: String,

    #[serde(rename = "type")]
    pub message_type: String,

    pub payload: RtdsPayload,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RtdsPayload {
    pub symbol: String,
    pub timestamp: i64,
    pub value: Value,

    #[serde(default)]
    pub full_accuracy_value: Option<String>,
}

impl RtdsMessage {
    pub fn is_update(&self) -> bool {
        self.message_type.eq_ignore_ascii_case("update")
    }

    pub fn has_topic(&self, topic: &str) -> bool {
        self.topic == topic
    }
}

impl RtdsPayload {
    pub fn raw_value_string(&self) -> String {
        match &self.value {
            Value::String(value) => value.clone(),
            Value::Number(value) => value.to_string(),
            other => other.to_string(),
        }
    }
}
