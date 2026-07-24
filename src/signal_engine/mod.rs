// src/signal_engine/mod.rs

pub mod filters;
pub mod publisher;
pub mod signal_builder;

pub use filters::{filter_signal_candidate, SignalFilterOutcome, SignalFilterRejection};

pub use publisher::SignalPublisher;

pub use signal_builder::{build_signal, build_signal_unchecked};
