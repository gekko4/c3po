// src/book_store/freshness.rs

use crate::types::Book;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BookFreshness {
    pub has_book: bool,
    pub has_usable_asks: bool,
    pub is_fresh: bool,
    pub is_usable: bool,
    pub age_ms: Option<i64>,
}

impl BookFreshness {
    pub fn missing() -> Self {
        Self {
            has_book: false,
            has_usable_asks: false,
            is_fresh: false,
            is_usable: false,
            age_ms: None,
        }
    }
}

pub fn check_book_freshness(
    book: Option<&Book>,
    now_ms: i64,
    max_book_age_ms: i64,
) -> BookFreshness {
    let Some(book) = book else {
        return BookFreshness::missing();
    };

    let age_ms = book.age_ms(now_ms);
    let has_usable_asks = book.has_usable_asks();
    let is_fresh = book.is_fresh(now_ms, max_book_age_ms);

    BookFreshness {
        has_book: true,
        has_usable_asks,
        is_fresh,
        is_usable: has_usable_asks && is_fresh,
        age_ms: Some(age_ms),
    }
}

pub fn is_book_fresh(book: Option<&Book>, now_ms: i64, max_book_age_ms: i64) -> bool {
    check_book_freshness(book, now_ms, max_book_age_ms).is_fresh
}

pub fn is_book_usable(book: Option<&Book>, now_ms: i64, max_book_age_ms: i64) -> bool {
    check_book_freshness(book, now_ms, max_book_age_ms).is_usable
}
