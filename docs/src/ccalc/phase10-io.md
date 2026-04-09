# Phase 10 — C-style I/O and `format` command

**Status:** Complete ✅  
**Version:** v0.14.0 / v0.14.0+001  
**Prerequisite:** Phase 9 (string types)

---

## What was implemented

### 10a — `fprintf` with format specifiers

`fprintf(fmt, arg1, arg2, ...)` built-in, interpreting a format string with
C-style conversion specifiers:

| Specifier | Meaning |
|-----------|---------|
| `%d`, `%i` | decimal integer (value truncated to i64) |
| `%f` | fixed-point decimal, default 6 places |
| `%e` | scientific notation (`1.23e+04`) |
| `%g` | shorter of `%f` and `%e` |
| `%s` | string (char array or string object) |
| `%%` | literal `%` |

Width, precision, and flags (`-`, `+`, `0`, space) follow C `printf`.

When there are more arguments than specifiers, the format string repeats for
remaining args (Octave behaviour).

`fprintf` is implemented as a case in `call_builtin` in `eval.rs` — not as a
REPL special case. Returns `Value::Void`; the REPL suppresses display for Void.

### 10b — `sprintf`

Same format engine (`format_printf` in `eval.rs`), but returns the formatted
result as `Value::Str` (char array) instead of printing.

Single-arg `sprintf(fmt)` processes escape sequences — this replaces the former
escape-only form.

### 10c — Precision system overhaul

The `p` / `p<N>` precision directive was **removed** from the language.
Replaced by the `format` command (v0.14.0+001).

### 10d — `format` command (v0.14.0+001)

MATLAB-compatible number display modes:

| Mode     | Description                                          |
|----------|------------------------------------------------------|
| `short`  | 5 significant digits, auto fixed/scientific (default) |
| `long`   | 15 significant digits                                |
| `shortE` | always scientific, 4 decimal places                  |
| `longE`  | always scientific, 14 decimal places                 |
| `shortG` | alias for `short`                                    |
| `longG`  | alias for `long`                                     |
| `bank`   | fixed 2 decimal places                               |
| `rat`    | rational approximation via continued fractions       |
| `hex`    | IEEE 754 double bit pattern (16 uppercase hex digits)|
| `+`      | sign only: `+`, `-`, or space                        |
| `compact`| suppress blank lines between outputs                 |
| `loose`  | add blank line after every output                    |
| `N`      | N decimal places (legacy custom mode)                |

`format` alone resets to `short`.

---

## Key implementation details

### `Value::Void`

New variant added to the `Value` enum in `env.rs`. Returned by side-effectful
functions (`fprintf`). The REPL checks for `Value::Void` and skips display.

### `format_printf` function

Core format engine in `eval.rs` (public). Processes a format string and a slice
of `Value` args, returns a `String`. Handles repeat-format semantics, all
specifiers, all flags.

### `FormatMode` enum

```rust
pub enum FormatMode {
    Short, Long, ShortE, LongE, ShortG, LongG,
    Bank, Rat, Hex, Plus, Custom(usize),
}
```

Added after `Base` enum in `eval.rs`. Default is `Custom(10)` (preserves
existing test baseline). `format` command in REPL resets to `Short`.

Changed public API signatures:
```rust
pub fn format_scalar(n: f64, base: Base, mode: &FormatMode) -> String
pub fn format_complex(re: f64, im: f64, mode: &FormatMode) -> String
pub fn format_value(v: &Value, base: Base, mode: &FormatMode) -> String
pub fn format_value_full(v: &Value, mode: &FormatMode) -> Option<String>
```

### `fmt_rat` — rational approximation

Continued fractions algorithm. Tolerance: 1e-6. Max denominator: 10,000.
Gives `355/113` for `pi`.

### `trim_sci` — exponent normalization

Normalizes Rust's `e0`/`e-3` format to MATLAB-style `e+00`/`e-03` with
two-digit exponent and explicit sign.

---

## Scope boundary

- **File I/O** (`fopen`/`fclose`/`fgetl`, `dlmread`/`dlmwrite`) — implemented in Phase 10.5.
- `fprintf(fd, ...)` with file descriptor — implemented in Phase 10.5.
- Complex matrices in `fprintf` — not supported (same boundary as Phase 8).

---

## Tests

- `eval_tests.rs`: 9 new `test_format_*` tests covering all modes.
- `repl_tests.rs`: updated harness (`FormatMode::default()` replaces `precision: usize`).
- Total: 427 tests passing (97 repl + 330 engine).
