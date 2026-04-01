# Phase 3 — Matrix Literals

**Version:** v0.8.0

## What was added

Matrix literals using Octave/MATLAB bracket syntax:

```
[1 2 3]          % row vector
[1; 2; 3]        % column vector
[1 2; 3 4]       % 2×2 matrix
```

Elements can be arbitrary expressions. Spaces and commas are both valid
element separators within a row.

## Arithmetic

- `scalar op matrix` and `matrix op scalar` — element-wise for `+`, `-`, `*`, `/`, `^`
- `matrix + matrix` and `matrix - matrix` — element-wise (shapes must match)
- `matrix * matrix` — not yet (Phase 4)

## Type system changes

`Value` enum introduced in `env.rs`:

```rust
pub enum Value {
    Scalar(f64),
    Matrix(ndarray::Array2<f64>),
}

pub type Env = HashMap<String, Value>;
```

`eval()` now returns `Result<Value, String>` (was `Result<f64, String>`).

## Display

Matrices print with right-aligned columns:

```
A =
   1   2
   3   4
```

The REPL prompt shows matrix size when `ans` is a matrix: `[ [2×2] ]: `.

Workspace save (`ws`) skips matrix variables — only scalars are persisted.

## Parser changes

New tokens: `LBracket` (`[`), `RBracket` (`]`), `Semicolon` (`;`).

New AST node: `Expr::Matrix(Vec<Vec<Expr>>)`.

`split_stmts()` in `repl.rs` is now bracket-depth-aware: a `;` inside
`[...]` is treated as a row separator by the parser, not a statement
separator by the REPL.

## Dependency added

`ndarray = "0.16"` in `crates/ccalc-engine/Cargo.toml`.
