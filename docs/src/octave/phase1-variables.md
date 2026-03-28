# Phase 1 — Variables and Assignment

**Goal**: `x = 5`, then `x + 1` → `6`

## What changes

### New `Value` type

Replace bare `f64` in the evaluator with an enum that can grow:

```rust
pub enum Value {
    Scalar(f64),
    // Matrix(Array2<f64>) — added in Phase 3
}
```

### Variable environment (`Env`)

```rust
pub type Env = HashMap<String, Value>;
```

`eval` gains a `&mut Env` parameter. The REPL holds one `Env` per session.

### Assignment statement

`x = expr` is a new statement type, separate from an expression:

```rust
pub enum Stmt {
    Assign(String, Expr),   // x = expr
    Expr(Expr),             // standalone expression
}
```

The parser's top-level entry point returns `Stmt` instead of `Expr`.

### Variable lookup in expressions

```rust
Expr::Var(String)   // new variant
```

When the evaluator encounters `Var(name)`, it looks up `name` in `Env`.
Unknown names produce an error (as before for unknown identifiers).

### New commands

| Command | Action |
|---|---|
| `who` | List all variables and their values |
| `clear x` | Delete variable `x` from `Env` |
| `clear` | Delete all variables |

### Variable-name-as-expression

Typing a variable name alone prints its value in Octave style:

```
>> x = 5
x = 5
>> x
x = 5
```

## Coexistence with existing features

- `acc` and `m1`–`m9` remain unchanged.
- `pi` and `e` continue to work as before (resolved before `Env` lookup).
- Partial expressions (`+ 5`, `* 2`) still use the accumulator.

## Key constraint

`eval` currently takes no mutable state. After this phase it takes
`&mut Env`. All call sites in `repl.rs` must be updated.
