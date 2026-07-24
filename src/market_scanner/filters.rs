// src/market_scanner/filters.rs

use crate::config::Config;
use crate::types::Market;

pub fn filter_market(market: &Market, config: &Config) -> bool {
    config.assets.is_enabled(&market.asset)
        && config.timeframes.is_enabled(&market.timeframe)
        && market.has_tokens()
        && market.has_valid_time_window()
}

pub fn filter_markets(markets: Vec<Market>, config: &Config) -> Vec<Market> {
    markets
        .into_iter()
        .filter(|market| filter_market(market, config))
        .collect()
}
