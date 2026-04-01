# Evaluator (`eval.rs`)

The evaluator walks an `Expr` AST and produces a `Value`, given a variable environment.

## Value type

```rust
// defined in env.rs
pub enum Value {
    Scalar(f64),
    Matrix(ndarray::Array2<f64>),
}

pub type Env = HashMap<String, Value>;
```

`Scalar` is the common case. `Matrix` was introduced in Phase 3.

## AST types

```rust
pub enum Expr {
    Number(f64),
    Var(String),
    UnaryMinus(Box<Expr>),
    BinOp(Box<Expr>, Op, Box<Expr>),
    Call(String, Vec<Expr>),
    Matrix(Vec<Vec<Expr>>),   // outer = rows, inner = elements per row
}

pub enum Op {
    Add, Sub, Mul, Div, Pow,
}
```

## `eval(expr, env)` semantics

| Variant | Semantics |
|---|---|
| `Number(n)` | Returns `Scalar(n)` |
| `Var(name)` | Clones value from `env`; `Err` if not defined |
| `UnaryMinus(e)` | Negates scalar or every matrix element |
| `BinOp(l, Div, r)` | `Err` if scalar divisor is `0.0` |
| `BinOp(l, Pow, r)` | Uses `f64::powf` for scalars; element-wise for `matrix ^ scalar` |
| `Call(name, args)` | Dispatches to built-in; currently all built-ins require scalar args |
| `Matrix(rows)` | Evaluates each element (must be scalar), builds `Array2` |

### Scalar × Matrix arithmetic

| Left | Op | Right | Result |
|---|---|---|---|
| `Scalar` | `+` `-` `*` `/` | `Matrix` | element-wise broadcast |
| `Matrix` | `+` `-` `*` `/` `^` | `Scalar` | element-wise broadcast |
| `Matrix` | `+` `-` | `Matrix` | element-wise (shapes must match) |
| `Matrix` | `*` `/` `^` | `Matrix` | error — matrix multiplication is Phase 4 |

## Number display

Three formatters serve different purposes:

### `format_value(v, precision, base)` — compact single-line

For scalars: respects the active display base (decimal, hex, bin, oct).
For matrices: returns `[N×M double]`.

Used in REPL prompts, `who` output, and assignment echo for scalars.

### `format_scalar(n, precision, base)` — scalar-only

Same as `format_value` for `Scalar` values. Used where a scalar is guaranteed
(prompt display, base conversion output).

### `format_value_full(v, precision)` — full multi-line

Returns `None` for scalars. For matrices returns a right-aligned column string:

```
   1   2   3
   4   5   6
```

Used by the REPL and pipe runner when printing matrix results.

### `format_number(n)` — internal / re-parsing

Always decimal, always round-trips through the parser. Used by `repl.rs`
when displaying partial-expression context (e.g. expanding variable names).

## `Base` enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Base { Dec, Hex, Bin, Oct }
```

Carried in `repl.rs` as the active display mode, passed to `format_scalar`
at output time, switched by `hex`, `dec`, `bin`, `oct` commands.
