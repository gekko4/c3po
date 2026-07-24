// src/telemetry/alerts.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthAlert {
    pub level: AlertLevel,
    pub code: String,
    pub message: String,
}

impl HealthAlert {
    pub fn info(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level: AlertLevel::Info,
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level: AlertLevel::Warning,
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn critical(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level: AlertLevel::Critical,
            code: code.into(),
            message: message.into(),
        }
    }
}
