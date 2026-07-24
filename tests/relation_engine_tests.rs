use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use c3po::relation_engine::{
    all_timeframe_pairs, build_package_candidate, decide_package_name, group_by_asset_and_end_ms,
    PackageName, RelationMarket,
};
use c3po::types::asset::Asset;
use c3po::types::timeframe::Timeframe;

#[derive(Debug, Clone)]
struct DummyMarket {
    asset: Asset,
    timeframe: Timeframe,
    slug: String,
    up_token_id: String,
    down_token_id: String,
    open_ms: i64,
    end_ms: i64,
    active: bool,
    closed: bool,
}

impl RelationMarket for DummyMarket {
    fn asset(&self) -> Asset {
        self.asset
    }

    fn timeframe(&self) -> Timeframe {
        self.timeframe
    }

    fn slug(&self) -> &str {
        &self.slug
    }

    fn up_token_id(&self) -> &str {
        &self.up_token_id
    }

    fn down_token_id(&self) -> &str {
        &self.down_token_id
    }

    fn open_ms(&self) -> i64 {
        self.open_ms
    }

    fn end_ms(&self) -> i64 {
        self.end_ms
    }

    fn active(&self) -> bool {
        self.active
    }

    fn closed(&self) -> bool {
        self.closed
    }
}

fn market(tf: Timeframe, slug: &str, up: &str, down: &str) -> DummyMarket {
    DummyMarket {
        asset: Asset::BTC,
        timeframe: tf,
        slug: slug.to_string(),
        up_token_id: up.to_string(),
        down_token_id: down.to_string(),
        open_ms: 1_000,
        end_ms: 10_000,
        active: true,
        closed: false,
    }
}

#[test]
fn groups_markets_by_asset_and_end_ms() {
    let markets = vec![
        market(Timeframe::FiveMin, "btc-5m", "up5", "down5"),
        market(Timeframe::FifteenMin, "btc-15m", "up15", "down15"),
    ];

    let groups = group_by_asset_and_end_ms(markets);

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].markets.len(), 2);
    assert_eq!(groups[0].key.asset, Asset::BTC);
    assert_eq!(groups[0].key.end_ms, 10_000);
}

#[test]
fn generates_all_different_timeframe_pairs() {
    let markets = vec![
        market(Timeframe::FiveMin, "btc-5m", "up5", "down5"),
        market(Timeframe::FifteenMin, "btc-15m", "up15", "down15"),
        market(Timeframe::OneHour, "btc-1h", "up1h", "down1h"),
    ];

    let groups = group_by_asset_and_end_ms(markets);
    let pairs = all_timeframe_pairs(&groups[0]);

    assert_eq!(pairs.len(), 3);

    assert_eq!(pairs[0].short.timeframe(), Timeframe::FiveMin);
    assert_eq!(pairs[0].long.timeframe(), Timeframe::FifteenMin);

    assert_eq!(pairs[1].short.timeframe(), Timeframe::FiveMin);
    assert_eq!(pairs[1].long.timeframe(), Timeframe::OneHour);

    assert_eq!(pairs[2].short.timeframe(), Timeframe::FifteenMin);
    assert_eq!(pairs[2].long.timeframe(), Timeframe::OneHour);
}

#[test]
fn short_ptb_greater_than_long_ptb_selects_long_up_plus_short_down() {
    let package = decide_package_name(dec!(101), dec!(100));

    assert_eq!(package, Some(PackageName::LongUpPlusShortDown));
}

#[test]
fn short_ptb_less_than_long_ptb_selects_long_down_plus_short_up() {
    let package = decide_package_name(dec!(99), dec!(100));

    assert_eq!(package, Some(PackageName::LongDownPlusShortUp));
}

#[test]
fn equal_ptb_returns_none() {
    let package = decide_package_name(dec!(100), dec!(100));

    assert_eq!(package, None);
}

#[test]
fn builds_long_up_plus_short_down_candidate() {
    let short = market(Timeframe::FiveMin, "btc-5m", "short-up", "short-down");
    let long = market(Timeframe::FifteenMin, "btc-15m", "long-up", "long-down");

    let pair = c3po::relation_engine::MarketPair { short, long };

    let candidate = build_package_candidate(&pair, dec!(101), dec!(100), 5_000).unwrap();

    assert_eq!(candidate.package_name, PackageName::LongUpPlusShortDown);
    assert_eq!(candidate.selected_long_token, "long-up");
    assert_eq!(candidate.selected_short_token, "short-down");
    assert_eq!(candidate.seconds_to_end, 5);
}

#[test]
fn builds_long_down_plus_short_up_candidate() {
    let short = market(Timeframe::FiveMin, "btc-5m", "short-up", "short-down");
    let long = market(Timeframe::FifteenMin, "btc-15m", "long-up", "long-down");

    let pair = c3po::relation_engine::MarketPair { short, long };

    let candidate = build_package_candidate(&pair, dec!(99), dec!(100), 5_000).unwrap();

    assert_eq!(candidate.package_name, PackageName::LongDownPlusShortUp);
    assert_eq!(candidate.selected_long_token, "long-down");
    assert_eq!(candidate.selected_short_token, "short-up");
    assert_eq!(candidate.seconds_to_end, 5);
}
