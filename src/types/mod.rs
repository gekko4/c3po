// src/types/mod.rs

//! Shared domain vocabulary for the C3PO engine.
//!
//! This module stays low-dependency and defines the common nouns used by:
//! - scanner
//! - RTDS
//! - PTB store
//! - registry
//! - relation engine
//! - evaluator
//! - depth engine
//! - signal engine
//! - persistence
//! - telemetry
//! - replay

pub mod asset;
pub mod book;
pub mod evaluation;
pub mod market;
pub mod package;
pub mod ptb;
pub mod signal;
pub mod tick;
pub mod timeframe;
pub mod token;

pub use asset::Asset;
pub use book::{Book, BookSide, Level};
pub use evaluation::{EvaluationClassification, EvaluationRow};
pub use market::{Market, MarketSlug};
pub use package::{LegRole, PackageCandidate, PackageName, PackageToken};
pub use ptb::{PriceToBeat, PtbSource, PtbStatus};
pub use signal::Signal;
pub use tick::{RtdsSymbol, Tick};
pub use timeframe::Timeframe;
pub use token::{ConditionId, TokenId};
