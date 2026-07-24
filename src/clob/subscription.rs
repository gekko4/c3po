// src/clob/subscription.rs

use std::collections::HashSet;

use crate::registry::MarketRegistry;
use crate::types::{Market, TokenId};

#[derive(Debug, Clone, Default)]
pub struct ClobSubscription {
    desired_tokens: HashSet<TokenId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SubscriptionDiff {
    pub subscribe: Vec<TokenId>,
    pub unsubscribe: Vec<TokenId>,
}

impl ClobSubscription {
    pub fn new() -> Self {
        Self {
            desired_tokens: HashSet::new(),
        }
    }

    pub fn set_desired_tokens<I>(&mut self, next_tokens: I) -> SubscriptionDiff
    where
        I: IntoIterator<Item = TokenId>,
    {
        let next: HashSet<TokenId> = next_tokens.into_iter().collect();

        let subscribe = next
            .difference(&self.desired_tokens)
            .cloned()
            .collect::<Vec<_>>();

        let unsubscribe = self
            .desired_tokens
            .difference(&next)
            .cloned()
            .collect::<Vec<_>>();

        self.desired_tokens = next;

        SubscriptionDiff {
            subscribe,
            unsubscribe,
        }
    }

    pub fn token_ids(&self) -> Vec<TokenId> {
        self.desired_tokens.iter().cloned().collect()
    }

    pub fn contains(&self, token_id: &TokenId) -> bool {
        self.desired_tokens.contains(token_id)
    }

    pub fn len(&self) -> usize {
        self.desired_tokens.len()
    }

    pub fn is_empty(&self) -> bool {
        self.desired_tokens.is_empty()
    }

    pub fn clear(&mut self) {
        self.desired_tokens.clear();
    }
}

pub fn desired_tokens_from_markets<'a, I>(markets: I) -> Vec<TokenId>
where
    I: IntoIterator<Item = &'a Market>,
{
    let mut tokens = HashSet::new();

    for market in markets {
        if market.up_token_id.is_non_empty() {
            tokens.insert(market.up_token_id.clone());
        }

        if market.down_token_id.is_non_empty() {
            tokens.insert(market.down_token_id.clone());
        }
    }

    tokens.into_iter().collect()
}

pub fn desired_tokens_from_registry(
    registry: &MarketRegistry,
    now_ms: i64,
    include_upcoming_ms: i64,
) -> Vec<TokenId> {
    let mut tokens = HashSet::new();

    for market in registry.live_markets(now_ms) {
        tokens.insert(market.up_token_id.clone());
        tokens.insert(market.down_token_id.clone());
    }

    for market in registry.upcoming_markets(now_ms) {
        if market.open_ms.saturating_sub(now_ms) <= include_upcoming_ms {
            tokens.insert(market.up_token_id.clone());
            tokens.insert(market.down_token_id.clone());
        }
    }

    tokens.into_iter().collect()
}
