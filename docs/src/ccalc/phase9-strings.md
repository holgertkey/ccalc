# Phase 9 — String Data Types

**Version:** v0.13.0

## What was added

### Two new Value variants

Two variants were added to the `Value` enum in `env.rs`:

```rust
pub enum Value {
    Scalar(f64),
    Matrix(Array2<f64>),
    Complex(f64, f64),
    /// Char array (single-quoted). Represents a 1×N row of characters.
    Str(String),
    /// String object (double-quoted). Scalar container of arbitrary text.
    StringObj(String),
}
```

`save_workspace` uses `as_scalar()` to filter variables before writing —
the new variants return `None` from `as_scalar()` and are automatically
skipped, matching the policy for matrices and complex numbers.

### Tokenizer changes — `'` disambiguation

The `'` character has two meanings:
- **Transpose operator** — when preceded by an rvalue context
- **Char array literal** — at the start of an expression or after an operator

The tokenizer tracks the last emitted token and applies this rule:

| Last token | `'` is |
|---|---|
| `Number`, `Ident`, `RParen`, `RBracket`, `Apostrophe`, `Str` | Transpose (`Token::Apostrophe`) |
| Anything else (including "nothing" = start of input) | Char array literal start |

When a char array literal is detected, the tokenizer consumes characters
until the next `'`. The sequence `''` (two consecutive single quotes)
represents a literal single-quote inside the string.

```
'hello'      →  Token::Str("hello")
'it''s ok'   →  Token::Str("it's ok")
x'           →  Ident("x")  Apostrophe       (transpose)
'A''         →  Str("A")    Apostrophe       (char array, then transpose)
```

### `"..."` string object tokens

A new arm handles double-quoted string objects. Escape sequences are
processed at tokenization time:

| Sequence | Result |
|---|---|
| `""` | Literal `"` |
| `\n` | Newline |
| `\t` | Tab |
| `\\` | Literal `\` |
| `\"` | Literal `"` |

### New AST nodes

```rust
pub enum Expr {
    // ...existing variants...
    /// Single-quoted char array literal.
    StrLiteral(String),
    /// Double-quoted string object literal.
    StringObjLiteral(String),
}
```

`parse_primary` handles `Token::Str` → `Expr::StrLiteral` and
`Token::StringObj` → `Expr::StringObjLiteral`. The existing postfix
transpose loop then applies: `'hello''` produces
`Expr::Transpose(Box::new(Expr::StrLiteral("hello")))`.

### Arithmetic on char arrays

`str_to_numeric(s: &str) -> Value` converts a char array to its numeric
representation before binary operations:

| Input length | Result |
|---|---|
| 0 | `Value::Matrix` 1×0 |
| 1 | `Value::Scalar(code)` |
| N | `Value::Matrix` 1×N |

In `eval_binop`, the arms for `Str` appear before all others:

```rust
(Value::Str(s), r) => eval_binop(str_to_numeric(&s), op, r),
(l, Value::Str(s)) => eval_binop(l, op, str_to_numeric(&s)),
```

This means `'abc' + 1` → `str_to_numeric("abc")` = `Matrix([97,98,99])`
→ `[98, 99, 100]`, and `'a' + 0` → `Scalar(97)` → `Scalar(97)`.

### String object operations

String objects support only `+` (concatenation) and `==`/`~=` (comparison).
All other operators return an error:

```rust
(Value::StringObj(a), Value::StringObj(b)) => match op {
    Op::Add   => Ok(Value::StringObj(a + &b)),
    Op::Eq    => Ok(Value::Scalar(bool_to_f64(a == b))),
    Op::NotEq => Ok(Value::Scalar(bool_to_f64(a != b))),
    _         => Err("Operator not supported on string objects"),
},
```

### New built-in functions

| Function | Arity | Description |
|---|---|---|
| `num2str` | 1 | Number → char array |
| `num2str` | 2 | Number → char array, N decimal digits |
| `str2num` | 1 | Char array → number, error on failure |
| `str2double` | 1 | Char array → number, NaN on failure |
| `strcat` | ≥2 | Concatenate strings |
| `strcmp` | 2 | Case-sensitive equality → 0/1 |
| `strcmpi` | 2 | Case-insensitive equality → 0/1 |
| `lower` | 1 | Lowercase |
| `upper` | 1 | Uppercase |
| `strtrim` | 1 | Strip whitespace |
| `strrep` | 3 | Find-and-replace |
| `sprintf` | 1 | Process escape sequences, return `Str` |
| `ischar` | 1 | 1 if `Str`, else 0 |
| `isstring` | 1 | 1 if `StringObj`, else 0 |

`length`, `numel`, and `size` were extended to handle both new variants:

- `length(Str(s))` → number of characters in `s`
- `length(StringObj(_))` → 1 (scalar element)
- `numel` and `size` follow the same pattern

### Helper functions

`string_arg(v, fname, pos)` extracts a string slice from `Str` or `StringObj`,
returning a descriptive error for any other type.

`process_escape_sequences(s)` handles `\n`, `\t`, `\\`, `\'`, `\"` —
factored out of `repl.rs` into `eval.rs` for use by `sprintf`.

### Display

- `format_value` returns the string content as-is for both variants.
- `format_value_full` returns `None` (strings are displayed inline like scalars).
- The REPL prompt shows the first 15 characters with surrounding quotes when `ans` is a string.
- `who` annotates type: `name [1×N char]` for `Str`, `name [string]` for `StringObj`.

### Exhaustiveness

All existing functions that matched `Value` variants (`eval_index`,
`resolve_dim`, `apply_elem`, `apply_reduction`, `apply_cumulative`,
`find_nonzero`, `scalar_arg`) were updated to handle `Str` and `StringObj` —
either with string-specific logic or a clear error message.

## What was not changed

- Matrix literals containing string elements are not yet supported
  (`['a', 'b']` as a char matrix). This requires a separate char-matrix
  representation and is deferred to a later phase.
- Workspace save/load for strings is intentionally skipped (same policy
  as matrices and complex).
- `strsplit` requires cell arrays (not yet implemented) and is deferred.
- The `split_stmts()` function in `repl.rs` already tracked single-quoted
  and double-quoted string boundaries (from earlier disambiguation work in
  Phase 4). No changes were needed there.
