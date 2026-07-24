// src/book_store/update.rs

use crate::types::{Book, BookSide, Level};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BookStoreUpdateStats {
    pub inserted: usize,
    pub replaced: usize,
    pub skipped: usize,
}

impl BookStoreUpdateStats {
    pub fn total_changed(&self) -> usize {
        self.inserted + self.replaced
    }
}

pub fn apply_book_snapshot(
    existing: Option<&mut Book>,
    mut incoming: Book,
) -> BookStoreUpdateStats {
    incoming.normalize();

    match existing {
        Some(existing_book) => {
            *existing_book = incoming;
            BookStoreUpdateStats {
                inserted: 0,
                replaced: 1,
                skipped: 0,
            }
        }
        None => BookStoreUpdateStats {
            inserted: 1,
            replaced: 0,
            skipped: 0,
        },
    }
}

pub fn apply_side_levels(
    book: &mut Book,
    side: BookSide,
    mut levels: Vec<Level>,
    exchange_ts_ms: Option<i64>,
    received_at_ms: i64,
) {
    levels.retain(|level| level.is_positive());

    match side {
        BookSide::Bid => {
            book.bids = levels;
        }
        BookSide::Ask => {
            book.asks = levels;
        }
    }

    book.exchange_ts_ms = exchange_ts_ms;
    book.received_at_ms = received_at_ms;
    book.normalize();
}
