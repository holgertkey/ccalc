# Evaluator (`eval.rs`)

The evaluator walks an `Expr` AST and produces an `f64` result, given a variable environment.

## AST types

```rust
pub enum Expr {
    Number(f64),
    Var(String),                     // variable lookup
    UnaryMinus(Box<Expr>),
    BinOp(Box<Expr>, Op, Box<Expr>),
    Call(String, Box<Expr>),         // single-argument function call
}

pub enum Op {
    Add, Sub, Mul, Div, Pow, Mod,
}
```

> `Call` currently accepts exactly one argument. Phase 2 (multi-argument
> functions) will extend this to `Call(String, Vec<Expr>)`.

## `eval(expr, env)` semantics

| Variant | Semantics |
|---|---|
| `Number(n)` | Returns `n` |
| `Var(name)` | Looks up `name` in `env`; `Err` if not defined |
| `UnaryMinus(e)` | Returns `-eval(e)` |
| `BinOp(l, Div, r)` | Returns `Err` if `r == 0.0` |
| `BinOp(l, Mod, r)` | Returns `Err` if `r == 0.0` |
| `BinOp(l, Pow, r)` | Uses `f64::powf` |
| `Call(name, arg)` | Dispatches to `f64` methods; `Err` on unknown name |

## Number display

Two formatters serve different purposes:

### `format_value(n, precision, base)` — user-facing output

Respects the active display base:

- `Base::Dec` → decimal with given precision, scientific for very large/small values
- `Base::Hex` / `Base::Bin` / `Base::Oct` → rounds to integer, formats with prefix

### `format_number(n)` — internal / re-parsing

Always decimal, always round-trips through the parser:

- Integers: no decimal point (`42`, not `42.0`)
- Floats: up to 10 significant fractional digits, trailing zeros trimmed
- Very large / very small: scientific notation to preserve value

This function is used by `repl.rs` when displaying partial-expression context.

## `Base` enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Base { Dec, Hex, Bin, Oct }
```

`Base` is carried in `repl.rs` as the active display mode, passed to
`format_value` at output time, and switched by commands (`hex`, `dec`, …).
