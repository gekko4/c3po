// src/runtime/tasks.rs

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use crate::book_store::BookStore;
use crate::clob::{desired_tokens_from_registry, run_clob_listener_loop, ClobSubscription};
use crate::config::Config;
use crate::market_scanner::{run_market_scanner_loop, MarketApiClient};
use crate::ptb_store::{
    apply_ptb_to_market, expected_rtds_symbol_for_market, match_market_to_tick, PtbStore,
};
use crate::registry::MarketRegistry;
use crate::rtds::run_rtds_listener_loop;
use crate::telemetry::{
    collect_telemetry_snapshot, run_health_checks, HealthCheckInput, TelemetryCounters,
};
use crate::tick_store::TickStore;
use crate::types::evaluation::EvaluationRow;

#[derive(Debug, Clone)]
pub struct RuntimeState {
    pub registry: Arc<RwLock<MarketRegistry>>,
    pub tick_store: Arc<RwLock<TickStore>>,
    pub ptb_store: Arc<RwLock<PtbStore>>,
    pub book_store: Arc<RwLock<BookStore>>,
    pub clob_subscription: Arc<RwLock<ClobSubscription>>,
    pub evaluation_rows: Arc<RwLock<Vec<EvaluationRow>>>,
    pub telemetry_counters: Arc<RwLock<TelemetryCounters>>,
}

impl RuntimeState {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(RwLock::new(MarketRegistry::new())),
            tick_store: Arc::new(RwLock::new(TickStore::new())),
            ptb_store: Arc::new(RwLock::new(PtbStore::new())),
            book_store: Arc::new(RwLock::new(BookStore::new())),
            clob_subscription: Arc::new(RwLock::new(ClobSubscription::new())),
            evaluation_rows: Arc::new(RwLock::new(Vec::new())),
            telemetry_counters: Arc::new(RwLock::new(TelemetryCounters::new())),
        }
    }
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RuntimeHandles {
    pub rtds_listener: JoinHandle<Result<()>>,
    pub market_scanner: JoinHandle<Result<()>>,
    pub ptb_matcher: JoinHandle<Result<()>>,
    pub clob_subscription_updater: JoinHandle<Result<()>>,
    pub clob_listener: JoinHandle<Result<()>>,
    pub health_monitor: JoinHandle<Result<()>>,
}

pub fn start_research_runtime(
    config: Config,
    market_client: MarketApiClient,
    state: RuntimeState,
) -> RuntimeHandles {
    let rtds_listener = {
        let rtds_config = config.rtds.clone();
        let tick_store = state.tick_store.clone();

        tokio::spawn(async move { run_rtds_listener_loop(rtds_config, tick_store).await })
    };

    let market_scanner = {
        let scanner_config = config.clone();
        let registry = state.registry.clone();

        tokio::spawn(async move {
            run_market_scanner_loop(market_client, registry, scanner_config).await
        })
    };

    let ptb_matcher = {
        let registry = state.registry.clone();
        let tick_store = state.tick_store.clone();
        let ptb_store = state.ptb_store.clone();
        let interval_ms = config.eval.discovery_interval_ms;

        tokio::spawn(async move {
            run_ptb_matcher_loop(registry, tick_store, ptb_store, interval_ms).await
        })
    };

    let clob_subscription_updater = {
        let registry = state.registry.clone();
        let book_store = state.book_store.clone();
        let clob_subscription = state.clob_subscription.clone();
        let max_subscription_tokens = config.clob.max_subscription_tokens;
        let interval_ms = config.eval.discovery_interval_ms;

        tokio::spawn(async move {
            run_clob_subscription_updater_loop(
                registry,
                book_store,
                clob_subscription,
                max_subscription_tokens,
                interval_ms,
            )
            .await
        })
    };

    let clob_listener = {
        let clob_config = config.clob.clone();
        let subscription = state.clob_subscription.clone();
        let book_store = state.book_store.clone();
        let telemetry_counters = state.telemetry_counters.clone();

        tokio::spawn(async move {
            {
                let mut counters = telemetry_counters.write().await;
                counters.increment_clob_reconnect();
            }

            run_clob_listener_loop(clob_config, subscription, move |books| {
                if let Ok(mut guard) = book_store.try_write() {
                    guard.apply_books(books);
                }
            })
            .await
        })
    };

    let health_monitor = {
        let health_config = config.clone();
        let registry = state.registry.clone();
        let ptb_store = state.ptb_store.clone();
        let book_store = state.book_store.clone();
        let evaluation_rows = state.evaluation_rows.clone();
        let telemetry_counters = state.telemetry_counters.clone();

        tokio::spawn(async move {
            run_health_monitor_loop(
                health_config,
                registry,
                ptb_store,
                book_store,
                evaluation_rows,
                telemetry_counters,
            )
            .await
        })
    };

    RuntimeHandles {
        rtds_listener,
        market_scanner,
        ptb_matcher,
        clob_subscription_updater,
        clob_listener,
        health_monitor,
    }
}

async fn run_ptb_matcher_loop(
    registry: Arc<RwLock<MarketRegistry>>,
    tick_store: Arc<RwLock<TickStore>>,
    ptb_store: Arc<RwLock<PtbStore>>,
    interval_ms: u64,
) -> Result<()> {
    loop {
        let now_ms = now_ms();

        let markets = {
            let registry_guard = registry.read().await;
            registry_guard
                .all_markets()
                .into_iter()
                .cloned()
                .collect::<Vec<_>>()
        };

        for market in markets {
            let ptb_result = {
                let symbol = expected_rtds_symbol_for_market(&market);
                let tick_guard = tick_store.read().await;
                let tick = tick_guard.get_tick(&symbol, market.open_ms);
                match_market_to_tick(&market, tick, now_ms)
            };

            let ptb = ptb_result.into_ptb();

            {
                let mut ptb_guard = ptb_store.write().await;
                ptb_guard.insert_ptb(ptb.clone());
            }

            {
                let mut registry_guard = registry.write().await;

                if let Some(market_mut) = registry_guard.get_market_mut(&market.slug) {
                    apply_ptb_to_market(market_mut, &ptb);
                }
            }
        }

        sleep(Duration::from_millis(interval_ms)).await;
    }
}

async fn run_clob_subscription_updater_loop(
    registry: Arc<RwLock<MarketRegistry>>,
    book_store: Arc<RwLock<BookStore>>,
    clob_subscription: Arc<RwLock<ClobSubscription>>,
    max_subscription_tokens: usize,
    interval_ms: u64,
) -> Result<()> {
    loop {
        let now_ms = now_ms();

        let mut desired_tokens = {
            let registry_guard = registry.read().await;
            desired_tokens_from_registry(&registry_guard, now_ms, 5 * 60 * 1_000)
        };

        desired_tokens.sort();

        if desired_tokens.len() > max_subscription_tokens {
            desired_tokens.truncate(max_subscription_tokens);
        }

        {
            let mut subscription_guard = clob_subscription.write().await;
            subscription_guard.set_desired_tokens(desired_tokens.clone());
        }

        {
            let mut book_store_guard = book_store.write().await;
            book_store_guard.retain_tokens(desired_tokens);
        }

        sleep(Duration::from_millis(interval_ms)).await;
    }
}

async fn run_health_monitor_loop(
    config: Config,
    registry: Arc<RwLock<MarketRegistry>>,
    ptb_store: Arc<RwLock<PtbStore>>,
    book_store: Arc<RwLock<BookStore>>,
    evaluation_rows: Arc<RwLock<Vec<EvaluationRow>>>,
    telemetry_counters: Arc<RwLock<TelemetryCounters>>,
) -> Result<()> {
    loop {
        if config.telemetry.enabled {
            let now_ms = now_ms();

            let snapshot = {
                let registry_guard = registry.read().await;
                let ptb_guard = ptb_store.read().await;
                let book_guard = book_store.read().await;
                let rows_guard = evaluation_rows.read().await;
                let counters_guard = telemetry_counters.read().await;

                collect_telemetry_snapshot(
                    &registry_guard,
                    &ptb_guard,
                    &book_guard,
                    &rows_guard,
                    *counters_guard,
                    now_ms,
                    config.eval.max_book_age_ms as i64,
                )
            };

            let alerts = run_health_checks(HealthCheckInput {
                config: &config,
                snapshot: &snapshot,
            });

            for alert in alerts {
                eprintln!("[{:?}] {}: {}", alert.level, alert.code, alert.message);
            }
        }

        sleep(Duration::from_millis(config.telemetry.health_interval_ms)).await;
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_millis() as i64
}
