// src/evaluator/evaluator_loop.rs

use crate::config::EvaluationConfig;
use crate::relation_engine::{PackageCandidate, PackageName as RelationPackageName};
use crate::types::book::Book;
use crate::types::evaluation::{EvaluationClassification, EvaluationRow};
use crate::types::package::PackageName as TypePackageName;
use crate::types::{MarketSlug, TokenId};

use super::top_book::{calculate_top_book_quote, TopBookQuote};

#[derive(Debug, Clone, Copy)]
pub struct BookPair<'a> {
    pub short_book: &'a Book,
    pub long_book: &'a Book,
}

pub fn evaluate_package_top_book(
    candidate: &PackageCandidate,
    short_book: &Book,
    long_book: &Book,
    eval_config: &EvaluationConfig,
    now_ms: i64,
) -> EvaluationRow {
    let seconds_to_end = seconds_to_end(candidate.end_ms, now_ms);

    if seconds_to_end < eval_config.min_seconds_to_end as i64 {
        return row_without_quote(
            candidate,
            now_ms,
            seconds_to_end,
            EvaluationClassification::ExpiredOrTooClose,
        );
    }

    if short_book.is_stale(now_ms, eval_config.max_book_age_ms as i64)
        || long_book.is_stale(now_ms, eval_config.max_book_age_ms as i64)
    {
        return row_without_quote(
            candidate,
            now_ms,
            seconds_to_end,
            EvaluationClassification::StaleBook,
        );
    }

    let quote = match calculate_top_book_quote(short_book, long_book, eval_config) {
        Some(quote) => quote,
        None => {
            return row_without_quote(
                candidate,
                now_ms,
                seconds_to_end,
                EvaluationClassification::NoUsableAsks,
            );
        }
    };

    let classification = classify_top_book(&quote);

    row_with_quote(
        candidate,
        now_ms,
        seconds_to_end,
        quote,
        classification,
    )
}

pub fn evaluate_package_without_books(
    candidate: &PackageCandidate,
    eval_config: &EvaluationConfig,
    now_ms: i64,
) -> EvaluationRow {
    let seconds_to_end = seconds_to_end(candidate.end_ms, now_ms);

    if seconds_to_end < eval_config.min_seconds_to_end as i64 {
        return row_without_quote(
            candidate,
            now_ms,
            seconds_to_end,
            EvaluationClassification::ExpiredOrTooClose,
        );
    }

    row_without_quote(
        candidate,
        now_ms,
        seconds_to_end,
        EvaluationClassification::NoUsableAsks,
    )
}

pub fn evaluate_candidate_batch<'a, I, F>(
    candidates: I,
    mut lookup_books: F,
    eval_config: &EvaluationConfig,
    now_ms: i64,
) -> Vec<EvaluationRow>
where
    I: IntoIterator<Item = &'a PackageCandidate>,
    F: FnMut(&PackageCandidate) -> Option<BookPair<'a>>,
{
    let mut rows = Vec::new();

    for candidate in candidates {
        let row = match lookup_books(candidate) {
            Some(book_pair) => evaluate_package_top_book(
                candidate,
                book_pair.short_book,
                book_pair.long_book,
                eval_config,
                now_ms,
            ),
            None => evaluate_package_without_books(candidate, eval_config, now_ms),
        };

        rows.push(row);
    }

    rows
}

fn classify_top_book(quote: &TopBookQuote) -> EvaluationClassification {
    if quote.is_under_trigger() {
        EvaluationClassification::TopOfBookOnlyUnderOne
    } else {
        EvaluationClassification::ExpectedAboveOne
    }
}

fn row_without_quote(
    candidate: &PackageCandidate,
    ts_ms: i64,
    seconds_to_end: i64,
    classification: EvaluationClassification,
) -> EvaluationRow {
    EvaluationRow::new(
        ts_ms,
        candidate.asset,
        candidate.short_tf,
        candidate.long_tf,
        to_market_slug(candidate.short_market_slug.clone()),
        to_market_slug(candidate.long_market_slug.clone()),
        Some(candidate.short_ptb),
        Some(candidate.long_ptb),
        Some(convert_package_name(candidate.package_name)),
        Some(to_token_id(candidate.selected_short_token.clone())),
        Some(to_token_id(candidate.selected_long_token.clone())),
        None,
        None,
        None,
        None,
        None,
        None,
        seconds_to_end,
        classification,
    )
}

fn row_with_quote(
    candidate: &PackageCandidate,
    ts_ms: i64,
    seconds_to_end: i64,
    quote: TopBookQuote,
    classification: EvaluationClassification,
) -> EvaluationRow {
    EvaluationRow::new(
        ts_ms,
        candidate.asset,
        candidate.short_tf,
        candidate.long_tf,
        to_market_slug(candidate.short_market_slug.clone()),
        to_market_slug(candidate.long_market_slug.clone()),
        Some(candidate.short_ptb),
        Some(candidate.long_ptb),
        Some(convert_package_name(candidate.package_name)),
        Some(to_token_id(candidate.selected_short_token.clone())),
        Some(to_token_id(candidate.selected_long_token.clone())),
        Some(quote.short_ask),
        Some(quote.long_ask),
        Some(quote.short_ask_size),
        Some(quote.long_ask_size),
        Some(quote.top_cost),
        Some(quote.edge),
        seconds_to_end,
        classification,
    )
}

fn seconds_to_end(end_ms: i64, now_ms: i64) -> i64 {
    end_ms.saturating_sub(now_ms).saturating_div(1_000).max(0)
}

fn convert_package_name(name: RelationPackageName) -> TypePackageName {
    match name {
        RelationPackageName::LongUpPlusShortDown => TypePackageName::LongUpPlusShortDown,
        RelationPackageName::LongDownPlusShortUp => TypePackageName::LongDownPlusShortUp,
    }
}

fn to_market_slug(value: String) -> MarketSlug {
    value.into()
}

fn to_token_id(value: String) -> TokenId {
    value.into()
}