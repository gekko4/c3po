use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::types::asset::Asset;
use crate::types::timeframe::Timeframe;

use super::deterministic::{decide_package_name, PackageName};
use super::grouping::RelationMarket;
use super::pair_generator::MarketPair;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageCandidate {
    pub asset: Asset,

    pub short_market_slug: String,
    pub long_market_slug: String,

    pub short_tf: Timeframe,
    pub long_tf: Timeframe,

    pub short_ptb: Decimal,
    pub long_ptb: Decimal,

    pub package_name: PackageName,

    pub selected_long_token: String,
    pub selected_short_token: String,

    pub end_ms: i64,
    pub seconds_to_end: i64,
}

pub fn build_package_candidate<M>(
    pair: &MarketPair<M>,
    short_ptb: Decimal,
    long_ptb: Decimal,
    now_ms: i64,
) -> Option<PackageCandidate>
where
    M: RelationMarket,
{
    if !pair.same_asset() || !pair.same_end_ms() || !pair.different_timeframe() {
        return None;
    }

    let package_name = decide_package_name(short_ptb, long_ptb)?;

    let (selected_long_token, selected_short_token) = match package_name {
        PackageName::LongUpPlusShortDown => {
            let long_up = pair.long.up_token_id().to_string();
            let short_down = pair.short.down_token_id().to_string();
            (long_up, short_down)
        }
        PackageName::LongDownPlusShortUp => {
            let long_down = pair.long.down_token_id().to_string();
            let short_up = pair.short.up_token_id().to_string();
            (long_down, short_up)
        }
    };

    Some(PackageCandidate {
        asset: pair.short.asset(),

        short_market_slug: pair.short.slug().to_string(),
        long_market_slug: pair.long.slug().to_string(),

        short_tf: pair.short.timeframe(),
        long_tf: pair.long.timeframe(),

        short_ptb,
        long_ptb,

        package_name,

        selected_long_token,
        selected_short_token,

        end_ms: pair.short.end_ms(),
        seconds_to_end: pair.seconds_to_end(now_ms),
    })
}
