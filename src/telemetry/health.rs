// src/telemetry/health.rs

use crate::config::Config;
use crate::telemetry::alerts::HealthAlert;
use crate::telemetry::metrics::TelemetrySnapshot;
use crate::types::Timeframe;

pub struct HealthCheckInput<'a> {
    pub config: &'a Config,
    pub snapshot: &'a TelemetrySnapshot,
}

pub fn run_health_checks(input: HealthCheckInput<'_>) -> Vec<HealthAlert> {
    let mut alerts = Vec::new();

    check_missing_configured_timeframes(input.config, input.snapshot, &mut alerts);
    check_higher_timeframe_ptbs(input.snapshot, &mut alerts);
    check_priced_pairs_only_5m_15m(input.snapshot, &mut alerts);
    check_book_health(input.snapshot, &mut alerts);
    check_depth_confirmation_absence(input.snapshot, &mut alerts);

    alerts
}

fn check_missing_configured_timeframes(
    config: &Config,
    snapshot: &TelemetrySnapshot,
    alerts: &mut Vec<HealthAlert>,
) {
    for timeframe in &config.timeframes.enabled {
        let discovered = snapshot
            .markets_discovered_by_timeframe
            .get(timeframe)
            .copied()
            .unwrap_or(0);

        if discovered == 0 {
            alerts.push(HealthAlert::warning(
                format!("missing_markets_{}", timeframe),
                format!(
                    "configured timeframe {} has zero discovered markets",
                    timeframe
                ),
            ));
        }
    }
}

fn check_higher_timeframe_ptbs(snapshot: &TelemetrySnapshot, alerts: &mut Vec<HealthAlert>) {
    for timeframe in [Timeframe::OneHour, Timeframe::FourHour, Timeframe::OneDay] {
        let captured_count: usize = snapshot
            .ptbs_captured_by_timeframe_and_asset
            .iter()
            .filter(|(key, _)| key.ends_with(&format!("-{}", timeframe)))
            .map(|(_, count)| *count)
            .sum();

        if captured_count == 0 {
            alerts.push(HealthAlert::warning(
                format!("zero_ptb_captures_{}", timeframe),
                format!("zero exact PTB captures for timeframe {}", timeframe),
            ));
        }
    }
}

fn check_priced_pairs_only_5m_15m(snapshot: &TelemetrySnapshot, alerts: &mut Vec<HealthAlert>) {
    if snapshot.priced_pairs_by_timeframe_pair.is_empty() {
        return;
    }

    let only_5m_15m = snapshot.priced_pairs_by_timeframe_pair.len() == 1
        && snapshot
            .priced_pairs_by_timeframe_pair
            .contains_key("5m-15m");

    if only_5m_15m {
        alerts.push(HealthAlert::warning(
            "priced_pairs_only_5m_15m",
            "priced pairs currently only include 5m-15m",
        ));
    }
}

fn check_book_health(snapshot: &TelemetrySnapshot, alerts: &mut Vec<HealthAlert>) {
    if snapshot.book_store_stats.stale_books > 0 {
        alerts.push(HealthAlert::warning(
            "stale_books_present",
            format!("{} books are stale", snapshot.book_store_stats.stale_books),
        ));
    }

    if snapshot.book_store_stats.books_without_usable_asks > 0 {
        alerts.push(HealthAlert::warning(
            "books_without_usable_asks",
            format!(
                "{} books have no usable asks",
                snapshot.book_store_stats.books_without_usable_asks
            ),
        ));
    }
}

fn check_depth_confirmation_absence(snapshot: &TelemetrySnapshot, alerts: &mut Vec<HealthAlert>) {
    let under_1_count: usize = snapshot
        .under_1_candidates_by_pair_type
        .values()
        .copied()
        .sum();

    let has_under_1 = under_1_count > 0;

    let has_max_edge = !snapshot.max_gross_edge_by_asset.is_empty();

    if has_under_1 && has_max_edge {
        alerts.push(HealthAlert::info(
            "under_1_candidates_seen",
            "under-1 candidates exist; depth confirmation should be monitored by signal/depth outputs",
        ));
    }
}
