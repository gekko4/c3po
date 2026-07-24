// src/market_scanner/client.rs

use anyhow::{anyhow, Context, Result};
use serde_json::Value;

use crate::market_scanner::parser::RawMarket;

#[derive(Debug, Clone)]
pub struct MarketApiClient {
    endpoint: String,
    http: reqwest::Client,
}

impl MarketApiClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            http: reqwest::Client::new(),
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub async fn fetch_raw_markets(&self) -> Result<Vec<RawMarket>> {
        let response = self
            .http
            .get(&self.endpoint)
            .send()
            .await
            .with_context(|| format!("failed to request markets from {}", self.endpoint))?
            .error_for_status()
            .with_context(|| format!("market endpoint returned error: {}", self.endpoint))?;

        let value: Value = response
            .json()
            .await
            .context("failed to deserialize market API response as JSON")?;

        decode_market_response(value)
    }
}

fn decode_market_response(value: Value) -> Result<Vec<RawMarket>> {
    if let Some(array) = value.as_array() {
        return decode_array(array.clone());
    }

    if let Some(array) = value.get("markets").and_then(|v| v.as_array()) {
        return decode_array(array.clone());
    }

    if let Some(array) = value.get("data").and_then(|v| v.as_array()) {
        return decode_array(array.clone());
    }

    Err(anyhow!(
        "unsupported market API response shape: expected array, markets[], or data[]"
    ))
}

fn decode_array(values: Vec<Value>) -> Result<Vec<RawMarket>> {
    values
        .into_iter()
        .map(|value| {
            serde_json::from_value::<RawMarket>(value).context("failed to decode raw market object")
        })
        .collect()
}
