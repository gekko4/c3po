use crate::types::timeframe::Timeframe;

use super::grouping::{MarketGroup, RelationMarket};

#[derive(Debug, Clone)]
pub struct MarketPair<M> {
    pub short: M,
    pub long: M,
}

impl<M> MarketPair<M>
where
    M: RelationMarket,
{
    pub fn pair_type(&self) -> (Timeframe, Timeframe) {
        (self.short.timeframe(), self.long.timeframe())
    }

    pub fn same_asset(&self) -> bool {
        self.short.asset() == self.long.asset()
    }

    pub fn same_end_ms(&self) -> bool {
        self.short.end_ms() == self.long.end_ms()
    }

    pub fn different_timeframe(&self) -> bool {
        self.short.timeframe() != self.long.timeframe()
    }

    pub fn seconds_to_end(&self, now_ms: i64) -> i64 {
        ((self.short.end_ms() - now_ms) / 1000).max(0)
    }
}

pub fn all_timeframe_pairs<M>(group: &MarketGroup<M>) -> Vec<MarketPair<M>>
where
    M: RelationMarket + Clone,
{
    let mut markets = group.markets.clone();

    markets.sort_by_key(|market| timeframe_seconds(market.timeframe()));

    let mut pairs = Vec::new();

    for i in 0..markets.len() {
        for j in (i + 1)..markets.len() {
            let short = markets[i].clone();
            let long = markets[j].clone();

            if short.asset() != long.asset() {
                continue;
            }

            if short.end_ms() != long.end_ms() {
                continue;
            }

            if short.timeframe() == long.timeframe() {
                continue;
            }

            pairs.push(MarketPair { short, long });
        }
    }

    pairs
}

pub fn timeframe_seconds(timeframe: Timeframe) -> u64 {
    match timeframe {
        Timeframe::FiveMin => 5 * 60,
        Timeframe::FifteenMin => 15 * 60,
        Timeframe::OneHour => 60 * 60,
        Timeframe::FourHour => 4 * 60 * 60,
        Timeframe::OneDay => 24 * 60 * 60,
    }
}