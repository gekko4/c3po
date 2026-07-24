// src/runtime/supervisor.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestartPolicy {
    pub enabled: bool,
    pub backoff_ms: u64,
}

impl RestartPolicy {
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            backoff_ms: 0,
        }
    }

    pub fn enabled(backoff_ms: u64) -> Self {
        Self {
            enabled: true,
            backoff_ms,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SupervisorState {
    pub rtds_restarts: usize,
    pub clob_restarts: usize,
    pub scanner_restarts: usize,
    pub ptb_matcher_restarts: usize,
    pub health_monitor_restarts: usize,
}

impl SupervisorState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_rtds_restart(&mut self) {
        self.rtds_restarts += 1;
    }

    pub fn record_clob_restart(&mut self) {
        self.clob_restarts += 1;
    }

    pub fn record_scanner_restart(&mut self) {
        self.scanner_restarts += 1;
    }

    pub fn record_ptb_matcher_restart(&mut self) {
        self.ptb_matcher_restarts += 1;
    }

    pub fn record_health_monitor_restart(&mut self) {
        self.health_monitor_restarts += 1;
    }
}
