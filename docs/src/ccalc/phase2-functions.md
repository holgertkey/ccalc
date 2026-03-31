# Phase 2 — Multi-argument Functions

**Status: ✅ Done** (v0.7.0+011)

**Goal**: `atan2(y, x)`, `mod(a, b)`, `max(a, b)`, `hypot(a, b)`, …

## What was implemented

### `Expr::Call` — variadic arguments

`Expr::Call(String, Box<Expr>)` was replaced by `Expr::Call(String, Vec<Expr>)`.
The parser now handles comma-separated argument lists:

```
fn()          →  passes ans (backward-compatible)
fn(x)         →  single argument
fn(a, b)      →  two arguments
fn(a, b, c)   →  three arguments (future use)
```

### `call_builtin` dispatcher

The inline `match name` in the evaluator was replaced by `call_builtin(name, args: &[f64])`,
which uses slice pattern matching to dispatch on both the function name and the
argument count:

```rust
("log", [x])        => x.log10()
("log", [x, base])  => x.log(*base)
```

This makes it trivial to add overloaded functions and keeps the evaluator clean.

### New one-argument functions

| Function  | Description                   |
|-----------|-------------------------------|
| `asin(x)` | Inverse sine (radians)        |
| `acos(x)` | Inverse cosine (radians)      |
| `atan(x)` | Inverse tangent (radians)     |
| `sign(x)` | Sign: −1.0, 0.0, or 1.0      |

### New two-argument functions

| Function       | Description                                             |
|----------------|---------------------------------------------------------|
| `atan2(y, x)`  | Four-quadrant inverse tangent (radians)                 |
| `mod(a, b)`    | Remainder, sign follows divisor (Octave/MATLAB `mod`)   |
| `rem(a, b)`    | Remainder, sign follows dividend (IEEE 754 truncation)  |
| `max(a, b)`    | Larger of two values                                    |
| `min(a, b)`    | Smaller of two values                                   |
| `hypot(a, b)`  | √(a²+b²), numerically stable                           |
| `log(x, base)` | Logarithm of x to an arbitrary base                    |

### `mod` vs `rem` — sign convention

```
mod( 10,  3)  →   1    rem( 10,  3)  →   1
mod(-1,   3)  →   2    rem(-1,   3)  →  -1
mod( 1,  -3)  →  -2    rem( 1,  -3)  →   1
```

`mod` is implemented as `a - b * floor(a / b)` — sign follows the divisor.
`rem` is implemented as `a - b * trunc(a / b)` — sign follows the dividend.

## Octave/MATLAB alignment

- `atan2`, `mod`, `rem`, `max`, `min`, `hypot` match Octave/MATLAB exactly.
- `log(x, base)` — Octave uses `log(x)` for the natural log; ccalc keeps
  `log(x)` as base-10 (legacy). Use `ln(x)` for natural log, or
  `log(x, e)` for the two-argument form.

## Example

See `examples/ac_impedance.ccalc` for a practical use of `hypot`, `atan2`,
`mod`, `max`, `min`, `log`, and `log(x, base)` in an AC circuit calculation.
