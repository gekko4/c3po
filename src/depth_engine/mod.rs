// src/depth_engine/mod.rs

pub mod result;
pub mod sizing;
pub mod walker;

pub use result::{DepthLeg, DepthResult, DepthStatus};
pub use sizing::{
    calculate_depth_for_size,
    find_max_size_under_threshold,
    DepthSizingConfig,
};
pub use walker::{walk_ask_book, WalkResult};