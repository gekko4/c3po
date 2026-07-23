// src/ptb_store/store.rs

use std::collections::HashMap;

use crate::types::market::Market;
use crate::types::ptb::{PriceToBeat, PtbStatus};
use crate::types::{MarketSlug};

#[derive(Debug, Clone, Default)]
pub struct PtbStore {
    by_market_slug: HashMap<MarketSlug, PriceToBeat>,
}

impl PtbStore {
    pub fn new() -> Self {
        Self {
            by_market_slug: HashMap::new(),
        }
    }

    pub fn insert_ptb(&mut self, ptb: PriceToBeat) -> Option<PriceToBeat> {
        self.by_market_slug.insert(ptb.market_slug.clone(), ptb)
    }

    pub fn insert_pending_for_market(&mut self, market: &Market) -> Option<PriceToBeat> {
        self.insert_ptb(PriceToBeat::pending(
            market.slug.clone(),
            market.asset,
            market.timeframe,
            market.open_ms,
        ))
    }

    pub fn insert_missing_for_market(
        &mut self,
        market: &Market,
        checked_at_ms: i64,
    ) -> Option<PriceToBeat> {
        self.insert_ptb(PriceToBeat::missing(
            market.slug.clone(),
            market.asset,
            market.timeframe,
            market.open_ms,
            checked_at_ms,
        ))
    }

    pub fn get_ptb_for_market(&self, market_slug: &MarketSlug) -> Option<&PriceToBeat> {
        self.by_market_slug.get(market_slug)
    }

    pub fn get_mut_ptb_for_market(
        &mut self,
        market_slug: &MarketSlug,
    ) -> Option<&mut PriceToBeat> {
        self.by_market_slug.get_mut(market_slug)
    }

    pub fn get_status_for_market(&self, market_slug: &MarketSlug) -> Option<PtbStatus> {
        self.get_ptb_for_market(market_slug).map(|ptb| ptb.status)
    }

    pub fn has_ptb_for_market(&self, market_slug: &MarketSlug) -> bool {
        self.by_market_slug.contains_key(market_slug)
    }

    pub fn has_exact_ptb_for_market(&self, market_slug: &MarketSlug) -> bool {
        self.get_ptb_for_market(market_slug)
            .map(|ptb| ptb.is_exact_rtds_open_tick())
            .unwrap_or(false)
    }

    pub fn value_for_market(
        &self,
        market_slug: &MarketSlug,
    ) -> Option<rust_decimal::Decimal> {
        self.get_ptb_for_market(market_slug)
            .and_then(|ptb| ptb.value())
    }

    pub fn remove_market(&mut self, market_slug: &MarketSlug) -> Option<PriceToBeat> {
        self.by_market_slug.remove(market_slug)
    }

    pub fn len(&self) -> usize {
        self.by_market_slug.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_market_slug.is_empty()
    }

    pub fn all(&self) -> impl Iterator<Item = &PriceToBeat> {
        self.by_market_slug.values()
    }

    pub fn captured_exact(&self) -> impl Iterator<Item = &PriceToBeat> {
        self.by_market_slug
            .values()
            .filter(|ptb| ptb.is_exact_rtds_open_tick())
    }

    pub fn pending(&self) -> impl Iterator<Item = &PriceToBeat> {
        self.by_market_slug
            .values()
            .filter(|ptb| ptb.is_pending())
    }

    pub fn missing(&self) -> impl Iterator<Item = &PriceToBeat> {
        self.by_market_slug
            .values()
            .filter(|ptb| ptb.is_missing())
    }

    pub fn invalid(&self) -> impl Iterator<Item = &PriceToBeat> {
        self.by_market_slug
            .values()
            .filter(|ptb| ptb.is_invalid())
    }
}