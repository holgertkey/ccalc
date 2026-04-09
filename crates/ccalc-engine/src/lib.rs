//! # ccalc-engine
//!
//! Core computation engine for [`ccalc`](https://github.com/holgertkey/ccalc).
//!
//! This crate provides the language pipeline:
//!
//! ```text
//! input string
//!     └─► tokenizer (parser::tokenize)
//!             └─► recursive-descent parser (parser::parse)  →  Stmt AST
//!                     └─► evaluator (eval::eval)  →  f64
//! ```
//!
//! It also hosts [`env`] (variable environment and workspace persistence),
//! and will grow to host the Octave/MATLAB compatibility layer in future phases.
//!
//! ## Modules
//!
//! - [`env`]    — [`Env`](env::Env) type, workspace save/load
//! - [`eval`]   — AST types, evaluator, number formatters, [`Base`](eval::Base)
//! - [`parser`] — tokenizer and recursive-descent parser, [`Stmt`](parser::Stmt)

pub mod env;
pub mod eval;
pub mod io;
pub mod parser;
