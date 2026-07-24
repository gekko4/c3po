// src/market_scanner/scheduler.rs

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

use crate::config::Config;
use crate::market_scanner::client::MarketApiClient;
use crate::market_scanner::filters::filter_markets;
use crate::market_scanner::parser::parse_markets;
use crate::registry::MarketRegistry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScanStats {
    pub raw_markets: usize,
    pub parsed_markets: usize,
    pub accepted_markets: usize,
    pub inserted_or_updated_markets: usize,
}

pub async fn scan_once(
    client: &MarketApiClient,
    registry: &mut MarketRegistry,
    config: &Config,
    now_ms: i64,
) -> Result<ScanStats> {
    let raw_markets = client
        .fetch_raw_markets()
        .await
        .with_context(|| format!("market scan failed for endpoint {}", client.endpoint()))?;

    let raw_count = raw_markets.len();

    let parsed = parse_markets(&raw_markets, now_ms).context("failed to parse raw markets")?;

    let parsed_count = parsed.len();

    let accepted = filter_markets(parsed, config);
    let accepted_count = accepted.len();

    let mut inserted_or_updated = 0usize;

    for market in accepted {
        registry.insert_market(market);
        inserted_or_updated += 1;
    }

    Ok(ScanStats {
        raw_markets: raw_count,
        parsed_markets: parsed_count,
        accepted_markets: accepted_count,
        inserted_or_updated_markets: inserted_or_updated,
    })
}

pub async fn run_market_scanner_loop(
    client: MarketApiClient,
    registry: Arc<RwLock<MarketRegistry>>,
    config: Config,
) -> Result<()> {
    loop {
        let now_ms = now_ms();

        {
            let mut registry_guard = registry.write().await;
            scan_once(&client, &mut registry_guard, &config, now_ms).await?;
        }

        sleep(Duration::from_millis(config.eval.discovery_interval_ms)).await;
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_millis() as i64
}
