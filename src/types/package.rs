// src/types/package.rs

use crate::types::{Asset, MarketSlug, Timeframe, TokenId};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Deterministic cover package name.
///
/// These are strategy labels, not execution instructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PackageName {
    LongUpPlusShortDown,
    LongDownPlusShortUp,
}

impl PackageName {
    pub fn as_str(self) -> &'static str {
        match self {
            PackageName::LongUpPlusShortDown => "LONG_UP_PLUS_SHORT_DOWN",
            PackageName::LongDownPlusShortUp => "LONG_DOWN_PLUS_SHORT_UP",
        }
    }
}

impl fmt::Display for PackageName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str((*self).as_str())
    }
}

/// Role *f a selected token in the package.*#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegRole {
    Short*arket,
    LongMarket,
}

/// One *elected buy leg of a deterministic*package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageToken {
    pu* role: LegRole,
    pub token_id: *okenId,
    pub market_slug: Marke*Slug,
    pub timeframe: Timeframe*
}

impl PackageToken {
    pub fn*new(
        role: LegRole,
      * token_id: TokenId,
        market*slug: MarketSlug,
        timefram*: Timeframe,
    ) -> Self {
     *  Self {
            role,
       *    token_id,
            market_s*ug,
            timeframe,
       *}
    }

    pub fn is_short_marke*_leg(&self) -> bool {
        self*role == LegRole::ShortMarket
    }*
    pub fn is_long_market_leg(&se*f) -> bool {
        self.role == *egRole::LongMarket
    }
}

/// Ca*didate emitted by relation_engine *efore book pricing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageCan*idate {
    pub asset: Asset,

    pub short_market_slug: MarketSlug,
    pub long_market_slug: MarketSlug,

    pub short_tf: Timeframe,
    pub long_tf: Timeframe,

    pub short_ptb: Decimal,
    pub long_ptb: Decimal,

    pub package_name: PackageName,

    pub selected_short_token: TokenId,
    pub selected_long_token: TokenId,

    pub end_ms: i64,
    pub seconds_to_end: i64,
}

impl PackageCandidate {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        asset: Asset,
        short_market_slug: MarketSlug,
        long_market_slug: MarketSlug,
        short_tf: Timeframe,
        long_tf: Timeframe,
        short_ptb: Decimal,
        long_ptb: Decimal,
        package_name: PackageName,
        selected_short_token: TokenId,
        selected_long_token: TokenId,
        end_ms: i64,
        now_ms: i64,
    ) -> Self {
        let seconds_to_end = end_ms.saturating_sub(now_ms) / 1_000;

        Self {
            asset,
            short_market_slug,
            long_market_slug,
            short_tf,
            long_tf,
            short_ptb,
            long_ptb,
            package_name,
            selected_short_token,
            selected_long_token,
            end_ms,
            seconds_to_end,
        }
    }

    pub fn pair_type(&self) -> String {
        format!("{}-{}", self.short_tf, self.long_tf)
    }

    pub fn tokens(&self) -> [&TokenId; 2] {
        [&self.selected_short_token, &self.selected_long_token]
    }

    pub fn selected_short_package_token(&self) -> PackageToken {
        PackageToken::new(
            LegRole::ShortMarket,
            self.selected_short_token.clone(),
            self.short_market_slug.clone(),
            self.short_tf,
        )
    }

    pub fn selected_long_package_token(&self) -> PackageToken {
        PackageToken::new(
            LegRole::LongMarket,
            self.selected_long_token.clone(),
            self.long_market_slug.clone(),
            self.long_tf,
        )
    }

    pub fn package_tokens(&self) -> [PackageToken; 2] {
        [
            self.selected_short_package_token(),
            self.selected_long_package_token(),
        ]
    }

    pub fn recompute_seconds_to_end(&mut self, now_ms: i64) {
        self.seconds_to_end = self.end_ms.saturating_sub(now_ms) / 1_000;
    }

    pub fn is_expired_or_settled(&self, now_ms: i64) -> bool {
        now_ms >= self.end_ms
    }
}