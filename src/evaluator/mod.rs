// src/evaluator/mod.rs

pub mod classification;
pub mod evaluator_loop;
pub mod top_book;

pub use classification::EvaluationClassification;

pub use evaluator_loop::{
    evaluate_candidate_batch,
    evaluate_package_top_book,
    evaluate_package_without_books,
    BookPair,
};

pub use top_book::{
    calculate_top_book_quote,
    TopBookQuote,
};