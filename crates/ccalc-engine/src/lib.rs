//! # ccalc-engine
//!
//! Core computation engine for [`ccalc`](https://github.com/holgertkey/ccalc).
//!
//! This crate provides the language pipeline:
//!
//! ```text
//! input string
//!     └─► tokenizer (parser::tokenize)
//!             └─► recursive-descent parser (parser::parse)  →  Expr AST
//!                     └─► evaluator (eval::eval)  →  f64
//! ```
//!
//! It also hosts [`memory`] (m1–m9 cells and persistence), and will grow
//! to host the Octave/MATLAB compatibility layer in future phases.
//!
//! ## Modules
//!
//! - [`eval`]   — AST types, evaluator, number formatters, [`Base`](eval::Base)
//! - [`parser`] — tokenizer and recursive-descent parser
//! - [`memory`] — memory cells, directives, config-file persistence

pub mod eval;
pub mod memory;
pub mod parser;
