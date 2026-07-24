// src/clob/message.rs

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClobRawMessage {
    #[serde(default, alias = "event_type", alias = "type")]
    pub event_type: Option<String>,

    #[serde(
        default,
        alias = "asset_id",
        alias = "assetId",
        alias = "token_id",
        alias = "tokenId"
    )]
    pub token_id: Option<String>,

    #[serde(default)]
    pub market: Option<String>,

    #[serde(default)]
    pub bids: Option<Vec<ClobBookLevel>>,

    #[serde(default)]
    pub asks: Option<Vec<ClobBookLevel>>,

    #[serde(default)]
    pub buys: Option<Vec<ClobBookLevel>>,

    #[serde(default)]
    pub sells: Option<Vec<ClobBookLevel>>,

    #[serde(default)]
    pub changes: Option<Vec<ClobPriceChange>>,

    #[serde(default, alias = "timestamp", alias = "ts", alias = "time")]
    pub timestamp: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClobBookLevel {
    #[serde(default, alias = "price", alias = "p")]
    pub price: Option<Value>,

    #[serde(default, alias = "size", alias = "s")]
    pub size: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClobPriceChange {
    #[serde(
        default,
        alias = "asset_id",
        alias = "assetId",
        alias = "token_id",
        alias = "tokenId"
    )]
    pub token_id: Option<String>,

    #[serde(default, alias = "side")]
    pub side: Option<String>,

    #[serde(default, alias = "price", alias = "p")]
    pub price: Option<Value>,

    #[serde(default, alias = "size", alias = "s")]
    pub size: Option<Value>,

    #[serde(default, alias = "timestamp", alias = "ts", alias = "time")]
    pub timestamp: Option<Value>,
}
