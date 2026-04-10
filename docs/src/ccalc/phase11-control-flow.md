# Phase 11 — Core Control Flow

**Version:** 0.15.x  
**Status:** ✅ Complete

## Motivation

Phase 10 (formatted output) made it practical to write loop scripts. Phase 11
adds the block statements that make those scripts possible: `if`, `for`,
`while`, `break`, `continue`, and compound assignment operators.

## Phase 11a — Multi-line input and if / for / while (v0.15.0)

### Architecture

A new `exec.rs` module houses `exec_stmts()`, which avoids circular dependency:
`parser.rs` → `eval.rs`, and `exec.rs` → both. The `Stmt` enum gains five
new variants:

```rust
pub enum Stmt {
    Assign(String, Expr),
    Expr(Expr),
    If {
        cond: Expr,
        body: Vec<(Stmt, bool)>,
        elseif_branches: Vec<(Expr, Vec<(Stmt, bool)>)>,
        else_body: Option<Vec<(Stmt, bool)>>,
    },
    For  { var: String, range_expr: Expr, body: Vec<(Stmt, bool)> },
    While { cond: Expr, body: Vec<(Stmt, bool)> },
    Break,
    Continue,
}
```

`exec_stmts` returns `Result<Option<Signal>, String>` where `Signal` is
`Break | Continue`. Loops catch these signals; a signal that escapes to the
top level is reported as an error.

### REPL buffering

`block_depth_delta(line)` returns `+1` for `if`/`for`/`while` and `-1` for
`end`. The REPL accumulates lines into `block_buf` while `block_depth > 0`,
then calls `parse_stmts()` + `exec_stmts()` when the block closes.

```
[ 0 ]:   for k = 1:3
  >>   fprintf('%d\n', k)
  >> end
1
2
3
```

### For loop semantics

The range expression is evaluated once before any iteration. Iteration is
column-by-column (Octave convention):

- Row vector (1×N): each element becomes a `Scalar`
- M×N matrix: each column becomes an M×1 `Matrix`

### is_truthy

```rust
fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Scalar(n)             => *n != 0.0 && !n.is_nan(),
        Value::Matrix(m)             => m.iter().all(|&x| x != 0.0 && !x.is_nan()),
        Value::Complex(re, im)       => *re != 0.0 || *im != 0.0,
        Value::Str(s) | Value::StringObj(s) => !s.is_empty(),
        Value::Void                  => false,
    }
}
```

### Known limitation

Single-line blocks (`if cond; body; end` on one line) are not supported.
Use multi-line form.

## Phase 11b — Compound assignment operators (v0.15.1)

Six new token variants are added to the lexer:

| Token        | Source |
|--------------|--------|
| `PlusEq`     | `+=`   |
| `MinusEq`    | `-=`   |
| `StarEq`     | `*=`   |
| `SlashEq`    | `/=`   |
| `PlusPlus`   | `++`   |
| `MinusMinus` | `--`   |

`try_parse_compound(tokens)` is called first in `parse()`. All forms desugar
to `Stmt::Assign` at parse time — no new AST nodes are needed.

| Input   | Desugared to         |
|---------|----------------------|
| `x += e` | `x = x + e`        |
| `x -= e` | `x = x - e`        |
| `x *= e` | `x = x * e`        |
| `x /= e` | `x = x / e`        |
| `x++`   | `x = x + 1`         |
| `x--`   | `x = x - 1`         |
| `++x`   | `x = x + 1`         |
| `--x`   | `x = x - 1`         |

The RHS of `op=` is a full expression (`parse_logical_or`), so
`x *= 2 + 3` desugars to `x = x * (2 + 3)`.

`is_partial()` was updated so `++x`/`--x` are not mistakenly treated as
partial binary expressions.

**Limitation:** `++`/`--` are statement-level only.

## Syntax aliases (v0.15.2)

| Alias | Equivalent |
|-------|-----------|
| `#`   | `%` (comment) |
| `!`   | `~` (logical NOT) |
| `!=`  | `~=` (not equal) |

`#` is recognised in the tokenizer, `strip_line_comment()`, and
`split_stmts()`. `!` and `!=` are emitted as `Token::Tilde` and
`Token::NotEq` respectively — no changes downstream.
