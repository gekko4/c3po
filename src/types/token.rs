// src/types/token.rs

use serde::{Deserialize, Serialize};
use std::fmt;

/// Polymarket outcome token ID wrapper.
///
/// Token IDs can be very large string values, so they should not be represented
/// as integers or confused with condition IDs or market slugs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TokenId(String);

impl TokenId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }
}

impl fmt::Display for TokenId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for TokenId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for TokenId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// Polymarket condition ID wrapper.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ConditionId(String);

impl ConditionId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }
}

impl fmt::Display for ConditionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for ConditionId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for ConditionId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}