use std::collections::HashMap;

use crate::types::asset::Asset;
use crate::types::timeframe::Timeframe;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GroupKey {
    pub asset: Asset,
    pub end_ms: i64,
}

#[derive(Debug, Clone)]
pub struct MarketGroup<M> {
    pub key: GroupKey,
    pub markets: Vec<M>,
}

pub trait RelationMarket {
    fn asset(&self) -> Asset;
    fn timeframe(&self) -> Timeframe;
    fn slug(&self) -> &str;
    fn up_token_id(&self) -> &str;
    fn down_token_id(&self) -> &str;
    fn open_ms(&self) -> i64;
    fn end_ms(&self) -> i64;
    fn active(&self) -> bool;
    fn closed(&self) -> bool;

    fn is_live_at(&self, now_ms: i64) -> bool {
        self.active() && !self.closed() && self.open_ms() <= now_ms && now_ms < self.end_ms()
    }
}

pub fn group_by_asset_and_end_ms<M>(markets: impl IntoIterator<Item = M>) -> Vec<MarketGroup<M>>
where
    M: RelationMarket,
{
    let mut groups: HashMap<GroupKey, Vec<M>> = HashMap::new();

    for market in markets {
        let key = GroupKey {
            asset: market.asset(),
            end_ms: market.end_ms(),
        };

        groups.entry(key).or_default().push(market);
    }

    let mut result: Vec<MarketGroup<M>> = groups
        .into_iter()
        .map(|(key, markets)| MarketGroup { key, markets })
        .collect();

    result.sort_by_key(|group| (group.key.asset, group.key.end_ms));
    result
}

pub fn live_markets_only<M>(markets: impl IntoIterator<Item = M>, now_ms: i64) -> Vec<M>
where
    M: RelationMarket,
{
    markets
        .into_iter()
        .filter(|market| market.is_live_at(now_ms))
        .collect()
}