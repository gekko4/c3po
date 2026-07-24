// src/registry/indexes.rs

use std::collections::{HashMap, HashSet};

use crate::types::{Asset, Market, MarketSlug, Timeframe, TokenId};

#[derive(Debug, Clone, Default)]
pub struct MarketIndexes {
    by_asset: HashMap<Asset, HashSet<MarketSlug>>,
    by_timeframe: HashMap<Timeframe, HashSet<MarketSlug>>,
    by_asset_and_end_ms: HashMap<(Asset, i64), HashSet<MarketSlug>>,
    by_token_id: HashMap<TokenId, MarketSlug>,
}

impl MarketIndexes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, market: &Market) {
        self.by_asset
            .entry(market.asset)
            .or_default()
            .insert(market.slug.clone());

        self.by_timeframe
            .entry(market.timeframe)
            .or_default()
            .insert(market.slug.clone());

        self.by_asset_and_end_ms
            .entry((market.asset, market.end_ms))
            .or_default()
            .insert(market.slug.clone());

        self.by_token_id
            .insert(market.up_token_id.clone(), market.slug.clone());

        self.by_token_id
            .insert(market.down_token_id.clone(), market.slug.clone());
    }

    pub fn remove(&mut self, market: &Market) {
        remove_from_set_map(&mut self.by_asset, &market.asset, &market.slug);
        remove_from_set_map(&mut self.by_timeframe, &market.timeframe, &market.slug);
        remove_from_set_map(
            &mut self.by_asset_and_end_ms,
            &(market.asset, market.end_ms),
            &market.slug,
        );

        self.by_token_id.remove(&market.up_token_id);
        self.by_token_id.remove(&market.down_token_id);
    }

    pub fn update(&mut self, old_market: &Market, new_market: &Market) {
        self.remove(old_market);
        self.insert(new_market);
    }

    pub fn slugs_by_asset(&self, asset: Asset) -> Vec<MarketSlug> {
        self.by_asset
            .get(&asset)
            .map(cloned_slugs)
            .unwrap_or_default()
    }

    pub fn slugs_by_timeframe(&self, timeframe: Timeframe) -> Vec<MarketSlug> {
        self.by_timeframe
            .get(&timeframe)
            .map(cloned_slugs)
            .unwrap_or_default()
    }

    pub fn slugs_by_asset_and_end_ms(&self, asset: Asset, end_ms: i64) -> Vec<MarketSlug> {
        self.by_asset_and_end_ms
            .get(&(asset, end_ms))
            .map(cloned_slugs)
            .unwrap_or_default()
    }

    pub fn slug_by_token_id(&self, token_id: &TokenId) -> Option<&MarketSlug> {
        self.by_token_id.get(token_id)
    }

    pub fn clear(&mut self) {
        self.by_asset.clear();
        self.by_timeframe.clear();
        self.by_asset_and_end_ms.clear();
        self.by_token_id.clear();
    }
}

fn cloned_slugs(set: &HashSet<MarketSlug>) -> Vec<MarketSlug> {
    set.iter().cloned().collect()
}

fn remove_from_set_map<K>(map: &mut HashMap<K, HashSet<MarketSlug>>, key: &K, slug: &MarketSlug)
where
    K: Eq + std::hash::Hash + Clone,
{
    let should_remove_key = match map.get_mut(key) {
        Some(set) => {
            set.remove(slug);
            set.is_empty()
        }
        None => false,
    };

    if should_remove_key {
        map.remove(key);
    }
}
