// src/runtime/mod.rs

pub mod shutdown;
pub mod supervisor;
pub mod tasks;

pub use shutdown::wait_for_shutdown_signal;

pub use supervisor::{RestartPolicy, SupervisorState};

pub use tasks::{start_research_runtime, RuntimeHandles, RuntimeState};
