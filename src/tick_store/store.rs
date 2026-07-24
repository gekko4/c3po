// src/tick_store/store.rs

use std::collections::HashMap;

use crate::types::tick::{RtdsSymbol, Tick};

#[derive(Debug, Clone, Default)]
pub struct TickStore {
    by_exact_key: HashMap<(RtdsSymbol, i64), Tick>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickStoreStats {
    pub total_ticks: usize,
}

impl TickStore {
    pub fn new() -> Self {
        Self {
            by_exact_key: HashMap::new(),
        }
    }

    pub fn insert_tick(&mut self, tick: Tick) -> Option<Tick> {
        let key = (tick.symbol.clone(), tick.timestamp_ms);
        self.by_exact_key.insert(key, tick)
    }

    pub fn get_tick(&self, symbol: &RtdsSymbol, timestamp_ms: i64) -> Option<&Tick> {
        self.by_exact_key.get(&(symbol.clone(), timestamp_ms))
    }

    pub fn get_tick_for_asset_open(&self, symbol: impl AsRef<str>, open_ms: i64) -> Option<&Tick> {
        let symbol = RtdsSymbol::normalized(symbol);
        self.get_tick(&symbol, open_ms)
    }

    pub fn has_tick(&self, symbol: &RtdsSymbol, timestamp_ms: i64) -> bool {
        self.by_exact_key
            .contains_key(&(symbol.clone(), timestamp_ms))
    }

    pub fn remove_tick(&mut self, symbol: &RtdsSymbol, timestamp_ms: i64) -> Option<Tick> {
        self.by_exact_key.remove(&(symbol.clone(), timestamp_ms))
    }

    pub fn latest_for_symbol(&self, symbol: &RtdsSymbol) -> Option<&Tick> {
        self.by_exact_key
            .values()
            .filter(|tick| tick.same_symbol(symbol))
            .max_by_key(|tick| tick.timestamp_ms)
    }

    pub fn ticks_for_symbol<'a>(
        &'a self,
        symbol: &'a RtdsSymbol,
    ) -> impl Iterator<Item = &'a Tick> {
        self.by_exact_key
            .values()
            .filter(move |tick| tick.same_symbol(symbol))
    }

    pub fn ticks_between<'a>(
        &'a self,
        symbol: &'a RtdsSymbol,
        start_ms: i64,
        end_ms: i64,
    ) -> impl Iterator<Item = &'a Tick> {
        self.by_exact_key.values().filter(move |tick| {
            tick.same_symbol(symbol) && tick.timestamp_ms >= start_ms && tick.timestamp_ms <= end_ms
        })
    }

    pub fn all(&self) -> impl Iterator<Item = &Tick> {
        self.by_exact_key.values()
    }

    pub fn len(&self) -> usize {
        self.by_exact_key.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_exact_key.is_empty()
    }

    pub fn clear(&mut self) {
        self.by_exact_key.clear();
    }

    pub fn stats(&self) -> TickStoreStats {
        TickStoreStats {
            total_ticks: self.len(),
        }
    }
}
