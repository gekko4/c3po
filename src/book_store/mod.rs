// src/book_store/mod.rs

pub mod freshness;
pub mod store;
pub mod update;

pub use freshness::{check_book_freshness, is_book_fresh, is_book_usable, BookFreshness};

pub use store::{BookPair, BookStore, BookStoreStats};

pub use update::{apply_book_snapshot, apply_side_levels, BookStoreUpdateStats};
