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
//!                     └─► evaluator (eval::eval)  →  Value
//! ```
//!
//! ## Modules
//!
//! - [`mod@env`]    — [`Env`](env::Env) type, [`Value`](env::Value) enum, workspace save/load
//! - [`eval`]   — AST types, evaluator, number formatters, [`Base`](eval::Base)
//! - [`parser`] — tokenizer and recursive-descent parser, [`Stmt`](parser::Stmt)
//! - [`exec`]   — block/loop/function executor, script search path
//! - [`io`]     — file descriptor table for `fopen`/`fclose`/`fgetl`/`fprintf`

#![warn(missing_docs)]

/// Variable environment, [`Value`](env::Value) type, and workspace persistence.
pub mod env;

/// AST node types ([`Expr`](eval::Expr), [`Op`](eval::Op)), evaluator, and number formatters.
pub mod eval;

/// Block statement executor: loops, functions, `run`/`source`, search path management.
pub mod exec;

/// File I/O context ([`IoContext`](io::IoContext)) for the REPL session.
pub mod io;

/// Tokenizer, recursive-descent parser, and [`Stmt`](parser::Stmt) AST.
pub mod parser;

#[cfg(feature = "json")]
pub(crate) mod json;

#[cfg(feature = "mat")]
pub(crate) mod mat;
