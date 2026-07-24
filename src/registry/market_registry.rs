// src/registry/market_registry.rs

use std::collections::HashMap;

use crate::registry::indexes::MarketIndexes;
use crate::registry::lifecycle::{is_expired_market, is_live_market, is_upcoming_market};
use crate::types::{Asset, Market, MarketSlug, Timeframe, TokenId};

#[derive(Debug, Clone, Default)]
pub struct MarketRegistry {
    markets: HashMap<MarketSlug, Market>,
    indexes: MarketIndexes,
}

impl MarketRegistry {
    pub fn new() -> Self {
        Self {
            markets: HashMap::new(),
            indexes: MarketIndexes::new(),
        }
    }

    pub fn insert_market(&mut self, market: Market) -> Option<Market> {
        let slug = market.slug.clone();

        let old = self.markets.insert(slug, market.clone());

        if let Some(old_market) = &old {
            self.indexes.update(old_market, &market);
        } else {
            self.indexes.insert(&market);
        }

        old
    }

    pub fn update_market(&mut self, market: Market) -> Option<Market> {
        self.insert_market(market)
    }

    pub fn remove_market(&mut self, slug: &MarketSlug) -> Option<Market> {
        let removed = self.markets.remove(slug);

        if let Some(market) = &removed {
            self.indexes.remove(market);
        }

        removed
    }

    pub fn get_market(&self, slug: &MarketSlug) -> Option<&Market> {
        self.markets.get(slug)
    }

    pub fn get_market_mut(&mut self, slug: &MarketSlug) -> Option<&mut Market> {
        self.markets.get_mut(slug)
    }

    pub fn contains_market(&self, slug: &MarketSlug) -> bool {
        self.markets.contains_key(slug)
    }

    pub fn all_markets(&self) -> Vec<&Market> {
        self.markets.values().collect()
    }

    pub fn live_markets(&self, now_ms: i64) -> Vec<&Market> {
        self.markets
            .values()
            .filter(|market| is_live_market(market, now_ms))
            .collect()
    }

    pub fn upcoming_markets(&self, now_ms: i64) -> Vec<&Market> {
        self.markets
            .values()
            .filter(|market| is_upcoming_market(market, now_ms))
            .collect()
    }

    pub fn expired_markets(&self, now_ms: i64) -> Vec<&Market> {
        self.markets
            .values()
            .filter(|market| is_expired_market(market, now_ms))
            .collect()
    }

    pub fn markets_by_asset(&self, asset: Asset) -> Vec<&Market> {
        self.indexes
            .slugs_by_asset(asset)
            .iter()
            .filter_map(|slug| self.markets.get(slug))
            .collect()
    }

    pub fn markets_by_timeframe(&self, timeframe: Timeframe) -> Vec<&Market> {
        self.indexes
            .slugs_by_timeframe(timeframe)
            .iter()
            .filter_map(|slug| self.markets.get(slug))
            .collect()
    }

    pub fn markets_by_asset_and_end(&self, asset: Asset, end_ms: i64) -> Vec<&Market> {
        self.indexes
            .slugs_by_asset_and_end_ms(asset, end_ms)
            .iter()
            .filter_map(|slug| self.markets.get(slug))
            .collect()
    }

    pub fn market_by_token_id(&self, token_id: &TokenId) -> Option<&Market> {
        let slug = self.indexes.slug_by_token_id(token_id)?;
        self.markets.get(slug)
    }

    pub fn markets_missing_ptb(&self) -> Vec<&Market> {
        self.markets
            .values()
            .filter(|market| {
                matches!(
                    market.price_to_beat_status.as_deref(),
                    None | Some("pending_ptb") | Some("missing_ptb") | Some("invalid_ptb")
                )
            })
            .collect()
    }

    pub fn markets_with_exact_ptb(&self) -> Vec<&Market> {
        self.markets
            .values()
            .filter(|market| market.has_captured_exact_ptb())
            .collect()
    }

    pub fn mark_market_pending_ptb(&mut self, slug: &MarketSlug) -> bool {
        match self.markets.get_mut(slug) {
            Some(market) => {
                market.mark_pending_ptb();
                true
            }
            None => false,
        }
    }

    pub fn mark_market_missing_ptb(&mut self, slug: &MarketSlug) -> bool {
        match self.markets.get_mut(slug) {
            Some(market) => {
                market.mark_missing_ptb();
                true
            }
            None => false,
        }
    }

    pub fn len(&self) -> usize {
        self.markets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.markets.is_empty()
    }

    pub fn clear(&mut self) {
        self.markets.clear();
        self.indexes.clear();
    }
}
