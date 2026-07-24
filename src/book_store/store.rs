// src/book_store/store.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::book_store::freshness::{check_book_freshness, BookFreshness};
use crate::book_store::update::BookStoreUpdateStats;
use crate::types::{Book, TokenId};

#[derive(Debug, Clone, Copy)]
pub struct BookPair<'a> {
    pub first: &'a Book,
    pub second: &'a Book,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BookStoreStats {
    pub total_books: usize,
    pub usable_books: usize,
    pub stale_books: usize,
    pub books_without_usable_asks: usize,
}

#[derive(Debug, Clone, Default)]
pub struct BookStore {
    books: HashMap<TokenId, Book>,
}

impl BookStore {
    pub fn new() -> Self {
        Self {
            books: HashMap::new(),
        }
    }

    pub fn insert_book(&mut self, mut book: Book) -> Option<Book> {
        book.normalize();
        self.books.insert(book.token_id.clone(), book)
    }

    pub fn apply_book(&mut self, book: Book) -> BookStoreUpdateStats {
        let token_id = book.token_id.clone();

        if self.books.contains_key(&token_id) {
            self.insert_book(book);
            BookStoreUpdateStats {
                inserted: 0,
                replaced: 1,
                skipped: 0,
            }
        } else {
            self.insert_book(book);
            BookStoreUpdateStats {
                inserted: 1,
                replaced: 0,
                skipped: 0,
            }
        }
    }

    pub fn apply_books<I>(&mut self, books: I) -> BookStoreUpdateStats
    where
        I: IntoIterator<Item = Book>,
    {
        let mut stats = BookStoreUpdateStats::default();

        for book in books {
            let update = self.apply_book(book);
            stats.inserted += update.inserted;
            stats.replaced += update.replaced;
            stats.skipped += update.skipped;
        }

        stats
    }

    pub fn get_book(&self, token_id: &TokenId) -> Option<&Book> {
        self.books.get(token_id)
    }

    pub fn get_book_mut(&mut self, token_id: &TokenId) -> Option<&mut Book> {
        self.books.get_mut(token_id)
    }

    pub fn contains_book(&self, token_id: &TokenId) -> bool {
        self.books.contains_key(token_id)
    }

    pub fn get_book_pair<'a>(
        &'a self,
        first_token: &TokenId,
        second_token: &TokenId,
    ) -> Option<BookPair<'a>> {
        let first = self.get_book(first_token)?;
        let second = self.get_book(second_token)?;

        Some(BookPair { first, second })
    }

    pub fn freshness(
        &self,
        token_id: &TokenId,
        now_ms: i64,
        max_book_age_ms: i64,
    ) -> BookFreshness {
        check_book_freshness(self.get_book(token_id), now_ms, max_book_age_ms)
    }

    pub fn is_usable(&self, token_id: &TokenId, now_ms: i64, max_book_age_ms: i64) -> bool {
        self.freshness(token_id, now_ms, max_book_age_ms).is_usable
    }

    pub fn remove_book(&mut self, token_id: &TokenId) -> Option<Book> {
        self.books.remove(token_id)
    }

    pub fn retain_tokens<I>(&mut self, desired_tokens: I)
    where
        I: IntoIterator<Item = TokenId>,
    {
        let desired = desired_tokens
            .into_iter()
            .collect::<std::collections::HashSet<_>>();
        self.books.retain(|token_id, _| desired.contains(token_id));
    }

    pub fn all(&self) -> impl Iterator<Item = &Book> {
        self.books.values()
    }

    pub fn len(&self) -> usize {
        self.books.len()
    }

    pub fn is_empty(&self) -> bool {
        self.books.is_empty()
    }

    pub fn clear(&mut self) {
        self.books.clear();
    }

    pub fn stats(&self, now_ms: i64, max_book_age_ms: i64) -> BookStoreStats {
        let mut stats = BookStoreStats {
            total_books: self.books.len(),
            usable_books: 0,
            stale_books: 0,
            books_without_usable_asks: 0,
        };

        for book in self.books.values() {
            if book.is_usable(now_ms, max_book_age_ms) {
                stats.usable_books += 1;
            }

            if book.is_stale(now_ms, max_book_age_ms) {
                stats.stale_books += 1;
            }

            if !book.has_usable_asks() {
                stats.books_without_usable_asks += 1;
            }
        }

        stats
    }
}
