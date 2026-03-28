# Engine Crate (`ccalc-engine`)

The `ccalc-engine` crate is a pure computation library with no I/O dependencies
beyond file access for memory persistence. It exposes three public modules.

## Public API

```rust
// Parse an expression string into an AST
pub fn parser::parse(input: &str, accumulator: f64) -> Result<Expr, String>

// Check whether input is a partial expression (starts with an operator)
pub fn parser::is_partial(input: &str) -> bool

// Evaluate an AST to a float
pub fn eval::eval(expr: &Expr) -> Result<f64, String>

// Format a number for user-facing display
pub fn eval::format_value(n: f64, precision: usize, base: Base) -> String

// Format a number for internal use (always decimal, for re-parsing)
pub fn eval::format_number(n: f64) -> String

// Memory cell store + persistence
pub struct memory::Memory { ... }

// Parse a trailing memory directive from an expression string
pub fn memory::extract_directive(input: &str) -> (&str, Option<Directive>)

// Replace m1..m9 references with their numeric values
pub fn memory::expand_memory_refs(input: &str, memory: &Memory) -> (String, Option<String>)
```

## Why a separate crate?

The engine crate provides a stable, testable boundary between computation logic
and the CLI. This separation makes it straightforward to:

- **Test** the parser and evaluator in isolation with 100+ unit tests.
- **Extend** for Octave/MATLAB compatibility without touching the CLI code.
- **Embed** the calculator in other tools or a future WASM target.

## Extending the engine

All Octave compatibility work (Phases 1–9) will be added to this crate.
The binary crate will remain a thin CLI wrapper.
