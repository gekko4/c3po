use std::env;
use std::fmt;
use std::str::FromStr;

use anyhow::{anyhow, bail, Context, Result};
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::types::asset::Asset;
use crate::types::timeframe::Timeframe;

#[derive(Debug, Clone)]
pub struct Config {
    pub assets: AssetConfig,
    pub timeframes: TimeframeConfig,
    pub pairs: PairConfig,
    pub eval: EvaluationConfig,
    pub rtds: RtdsConfig,
    pub clob: ClobConfig,
    pub storage: StorageConfig,
    pub telemetry: TelemetryConfig,
}

#[derive(Debug, Clone)]
pub struct AssetConfig {
    pub enabled: Vec<Asset>,
}

#[derive(Debug, Clone)]
pub struct TimeframeConfig {
    pub enabled: Vec<Timeframe>,
}

#[derive(Debug, Clone)]
pub struct PairConfig {
    pub allowed: Vec<PairType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PairType {
    pub short: Timeframe,
    pub long: Timeframe,
}

#[derive(Debug, Clone)]
pub struct EvaluationConfig {
    pub trigger_buffer: Decimal,
    pub execution_buffer: Decimal,
    pub min_seconds_to_end: u64,
    pub max_book_age_ms: u64,
    pub min_top_book_size: Decimal,
    pub min_depth_confirmed_size: Decimal,
    pub min_gross_edge: Decimal,
    pub eval_interval_ms: u64,
    pub discovery_interval_ms: u64,
}

#[derive(Debug, Clone)]
pub struct RtdsConfig {
    pub websocket_url: String,
    pub topic: String,
    pub reconnect_backoff_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ClobConfig {
    pub websocket_url: String,
    pub max_subscription_tokens: usize,
    pub reconnect_backoff_ms: u64,
}

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub data_dir: String,
    pub ticks_dir: String,
    pub ptbs_dir: String,
    pub books_dir: String,
    pub evaluations_dir: String,
    pub opportunities_dir: String,
    pub replay_dir: String,
}

#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub health_interval_ms: u64,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_file =
            env::var("C3PO_CONFIG_FILE").unwrap_or_else(|_| "config/default.toml".to_string());

        Self::from_file(&config_file)
    }

    pub fn from_file(path: &str) -> Result<Self> {
        let raw = config::Config::builder()
            .add_source(config::File::with_name(path).required(true))
            .add_source(config::File::with_name("config/local").required(false))
            .add_source(
                config::Environment::with_prefix("C3PO")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()
            .with_context(|| format!("failed to build config from {path}"))?
            .try_deserialize::<RawConfig>()
            .context("failed to deserialize config")?;

        let config = Self::from_raw(raw)?;
        config.validate()?;

        Ok(config)
    }

    fn from_raw(raw: RawConfig) -> Result<Self> {
        let assets = AssetConfig {
            enabled: raw
                .assets
                .enabled
                .iter()
                .map(|s| parse_asset(s))
                .collect::<Result<Vec<_>>>()?,
        };

        let timeframes = TimeframeConfig {
            enabled: raw
                .timeframes
                .enabled
                .iter()
                .map(|s| parse_timeframe(s))
                .collect::<Result<Vec<_>>>()?,
        };

        let pairs = PairConfig {
            allowed: raw
                .pairs
                .allowed
                .iter()
                .map(|p| {
                    Ok(PairType {
                        short: parse_timeframe(&p.short)?,
                        long: parse_timeframe(&p.long)?,
                    })
                })
                .collect::<Result<Vec<_>>>()?,
        };

        let eval = EvaluationConfig {
            trigger_buffer: parse_decimal(&raw.eval.trigger_buffer, "eval.trigger_buffer")?,
            execution_buffer: parse_decimal(&raw.eval.execution_buffer, "eval.execution_buffer")?,
            min_seconds_to_end: raw.eval.min_seconds_to_end,
            max_book_age_ms: raw.eval.max_book_age_ms,
            min_top_book_size: parse_decimal(&raw.eval.min_top_book_size, "eval.min_top_book_size")?,
            min_depth_confirmed_size: parse_decimal(
                &raw.eval.min_depth_confirmed_size,
                "eval.min_depth_confirmed_size",
            )?,
            min_gross_edge: parse_decimal(&raw.eval.min_gross_edge, "eval.min_gross_edge")?,
            eval_interval_ms: raw.eval.eval_interval_ms,
            discovery_interval_ms: raw.eval.discovery_interval_ms,
        };

        Ok(Self {
            assets,
            timeframes,
            pairs,
            eval,
            rtds: raw.rtds,
            clob: raw.clob,
            storage: raw.storage,
            telemetry: raw.telemetry,
        })
    }

    pub fn validate(&self) -> Result<()> {
        self.assets.validate()?;
        self.timeframes.validate()?;
        self.pairs.validate(&self.timeframes)?;
        self.eval.validate()?;
        self.rtds.validate()?;
        self.clob.validate()?;
        self.storage.validate()?;
        self.telemetry.validate()?;

        Ok(())
    }
}

impl AssetConfig {
    pub fn validate(&self) -> Result<()> {
        if self.enabled.is_empty() {
            bail!("assets.enabled cannot be empty");
        }

        if has_duplicates(&self.enabled) {
            bail!("assets.enabled contains duplicates");
        }

        Ok(())
    }

    pub fn is_enabled(&self, asset: &Asset) -> bool {
        self.enabled.contains(asset)
    }
}

impl TimeframeConfig {
    pub fn validate(&self) -> Result<()> {
        if self.enabled.is_empty() {
            bail!("timeframes.enabled cannot be empty");
        }

        if has_duplicates(&self.enabled) {
            bail!("timeframes.enabled contains duplicates");
        }

        Ok(())
    }

    pub fn is_enabled(&self, timeframe: &Timeframe) -> bool {
        self.enabled.contains(timeframe)
    }
}

impl PairConfig {
    pub fn validate(&self, timeframes: &TimeframeConfig) -> Result<()> {
        if self.allowed.is_empty() {
            bail!("pairs.allowed cannot be empty");
        }

        if has_duplicates(&self.allowed) {
            bail!("pairs.allowed contains duplicates");
        }

        for pair in &self.allowed {
            if pair.short == pair.long {
                bail!("invalid pair {pair}: short and long timeframes cannot be equal");
            }

            if !timeframes.is_enabled(&pair.short) {
                bail!("invalid pair {pair}: short timeframe is not enabled");
            }

            if !timeframes.is_enabled(&pair.long) {
                bail!("invalid pair {pair}: long timeframe is not enabled");
            }

            if timeframe_seconds(&pair.short) >= timeframe_seconds(&pair.long) {
                bail!("invalid pair {pair}: short timeframe must be shorter than long timeframe");
            }
        }

        Ok(())
    }

    pub fn is_allowed(&self, short: &Timeframe, long: &Timeframe) -> bool {
        self.allowed
            .iter()
            .any(|p| &p.short == short && &p.long == long)
    }
}

impl EvaluationConfig {
    pub fn validate(&self) -> Result<()> {
        validate_decimal_range(
            self.trigger_buffer,
            "eval.trigger_buffer",
            Decimal::ZERO,
            Decimal::ONE,
        )?;

        validate_decimal_range(
            self.execution_buffer,
            "eval.execution_buffer",
            Decimal::ZERO,
            Decimal::ONE,
        )?;

        if self.min_seconds_to_end == 0 {
            bail!("eval.min_seconds_to_end must be greater than 0");
        }

        if self.max_book_age_ms == 0 {
            bail!("eval.max_book_age_ms must be greater than 0");
        }

        if self.min_top_book_size <= Decimal::ZERO {
            bail!("eval.min_top_book_size must be greater than 0");
        }

        if self.min_depth_confirmed_size <= Decimal::ZERO {
            bail!("eval.min_depth_confirmed_size must be greater than 0");
        }

        if self.min_gross_edge < Decimal::ZERO {
            bail!("eval.min_gross_edge cannot be negative");
        }

        if self.eval_interval_ms == 0 {
            bail!("eval.eval_interval_ms must be greater than 0");
        }

        if self.discovery_interval_ms == 0 {
            bail!("eval.discovery_interval_ms must be greater than 0");
        }

        Ok(())
    }
}

impl RtdsConfig {
    pub fn validate(&self) -> Result<()> {
        if self.websocket_url.trim().is_empty() {
            bail!("rtds.websocket_url cannot be empty");
        }

        if self.topic.trim().is_empty() {
            bail!("rtds.topic cannot be empty");
        }

        if self.reconnect_backoff_ms == 0 {
            bail!("rtds.reconnect_backoff_ms must be greater than 0");
        }

        Ok(())
    }
}

impl ClobConfig {
    pub fn validate(&self) -> Result<()> {
        if self.websocket_url.trim().is_empty() {
            bail!("clob.websocket_url cannot be empty");
        }

        if self.max_subscription_tokens == 0 {
            bail!("clob.max_subscription_tokens must be greater than 0");
        }

        if self.reconnect_backoff_ms == 0 {
            bail!("clob.reconnect_backoff_ms must be greater than 0");
        }

        Ok(())
    }
}

impl StorageConfig {
    pub fn validate(&self) -> Result<()> {
        let paths = [
            ("storage.data_dir", &self.data_dir),
            ("storage.ticks_dir", &self.ticks_dir),
            ("storage.ptbs_dir", &self.ptbs_dir),
            ("storage.books_dir", &self.books_dir),
            ("storage.evaluations_dir", &self.evaluations_dir),
            ("storage.opportunities_dir", &self.opportunities_dir),
            ("storage.replay_dir", &self.replay_dir),
        ];

        for (name, value) in paths {
            if value.trim().is_empty() {
                bail!("{name} cannot be empty");
            }
        }

        Ok(())
    }
}

impl TelemetryConfig {
    pub fn validate(&self) -> Result<()> {
        if self.enabled && self.health_interval_ms == 0 {
            bail!("telemetry.health_interval_ms must be greater than 0 when telemetry is enabled");
        }

        Ok(())
    }
}

impl fmt::Display for PairType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let short = format_timeframe(&self.short);
        let long = format_timeframe(&self.long);
        write!(f, "{short}-{long}")
    }
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    assets: RawAssetConfig,
    timeframes: RawTimeframeConfig,
    pairs: RawPairConfig,
    eval: RawEvaluationConfig,
    rtds: RtdsConfig,
    clob: ClobConfig,
    storage: StorageConfig,
    telemetry: TelemetryConfig,
}

#[derive(Debug, Deserialize)]
struct RawAssetConfig {
    enabled: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawTimeframeConfig {
    enabled: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawPairConfig {
    allowed: Vec<RawPairType>,
}

#[derive(Debug, Deserialize)]
struct RawPairType {
    short: String,
    long: String,
}

#[derive(Debug, Deserialize)]
struct RawEvaluationConfig {
    trigger_buffer: String,
    execution_buffer: String,
    min_seconds_to_end: u64,
    max_book_age_ms: u64,
    min_top_book_size: String,
    min_depth_confirmed_size: String,
    min_gross_edge: String,
    eval_interval_ms: u64,
    discovery_interval_ms: u64,
}

fn parse_asset(value: &str) -> Result<Asset> {
    match value.trim().to_ascii_uppercase().as_str() {
        "BTC" => Ok(Asset::BTC),
        "ETH" => Ok(Asset::ETH),
        "SOL" => Ok(Asset::SOL),
        "XRP" => Ok(Asset::XRP),
        other => Err(anyhow!("unknown asset: {other}")),
    }
}

fn parse_timeframe(value: &str) -> Result<Timeframe> {
    match value.trim().to_ascii_lowercase().as_str() {
        "5m" | "five_min" | "fivemin" => Ok(Timeframe::FiveMin),
        "15m" | "fifteen_min" | "fifteenmin" => Ok(Timeframe::FifteenMin),
        "1h" | "one_hour" | "onehour" => Ok(Timeframe::OneHour),
        "4h" | "four_hour" | "fourhour" => Ok(Timeframe::FourHour),
        "1d" | "one_day" | "oneday" => Ok(Timeframe::OneDay),
        other => Err(anyhow!("unknown timeframe: {other}")),
    }
}

fn format_timeframe(timeframe: &Timeframe) -> &'static str {
    match timeframe {
        Timeframe::FiveMin => "5m",
        Timeframe::FifteenMin => "15m",
        Timeframe::OneHour => "1h",
        Timeframe::FourHour => "4h",
        Timeframe::OneDay => "1d",
    }
}

fn timeframe_seconds(timeframe: &Timeframe) -> u64 {
    match timeframe {
        Timeframe::FiveMin => 5 * 60,
        Timeframe::FifteenMin => 15 * 60,
        Timeframe::OneHour => 60 * 60,
        Timeframe::FourHour => 4 * 60 * 60,
        Timeframe::OneDay => 24 * 60 * 60,
    }
}

fn parse_decimal(value: &str, field_name: &str) -> Result<Decimal> {
    Decimal::from_str(value)
        .with_context(|| format!("failed to parse {field_name} as decimal: {value}"))
}

fn validate_decimal_range(
    value: Decimal,
    field_name: &str,
    min: Decimal,
    max: Decimal,
) -> Result<()> {
    if value < min || value >= max {
        bail!("{field_name} must be >= {min} and < {max}");
    }

    Ok(())
}

fn has_duplicates<T: PartialEq>(items: &[T]) -> bool {
    for i in 0..items.len() {
        for j in (i + 1)..items.len() {
            if items[i] == items[j] {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_asset_strings() {
        assert_eq!(parse_asset("BTC").unwrap(), Asset::BTC);
        assert_eq!(parse_asset("eth").unwrap(), Asset::ETH);
        assert_eq!(parse_asset("sol").unwrap(), Asset::SOL);
        assert_eq!(parse_asset("xrp").unwrap(), Asset::XRP);
    }

    #[test]
    fn parses_timeframe_strings() {
        assert_eq!(parse_timeframe("5m").unwrap(), Timeframe::FiveMin);
        assert_eq!(parse_timeframe("15m").unwrap(), Timeframe::FifteenMin);
        assert_eq!(parse_timeframe("1h").unwrap(), Timeframe::OneHour);
        assert_eq!(parse_timeframe("4h").unwrap(), Timeframe::FourHour);
        assert_eq!(parse_timeframe("1d").unwrap(), Timeframe::OneDay);
    }

    #[test]
    fn rejects_invalid_pair_order() {
        let timeframes = TimeframeConfig {
            enabled: vec![
                Timeframe::FiveMin,
                Timeframe::FifteenMin,
                Timeframe::OneHour,
            ],
        };

        let pairs = PairConfig {
            allowed: vec![PairType {
                short: Timeframe::OneHour,
                long: Timeframe::FiveMin,
            }],
        };

        assert!(pairs.validate(&timeframes).is_err());
    }

    #[test]
    fn accepts_valid_pair_order() {
        let timeframes = TimeframeConfig {
            enabled: vec![
                Timeframe::FiveMin,
                Timeframe::FifteenMin,
                Timeframe::OneHour,
            ],
        };

        let pairs = PairConfig {
            allowed: vec![
                PairType {
                    short: Timeframe::FiveMin,
                    long: Timeframe::FifteenMin,
                },
                PairType {
                    short: Timeframe::FifteenMin,
                    long: Timeframe::OneHour,
                },
            ],
        };

        assert!(pairs.validate(&timeframes).is_ok());
    }
}