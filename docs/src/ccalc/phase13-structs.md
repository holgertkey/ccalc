# Phase 13 — Structs

**Version:** 0.19.0
**Status:** Complete (13a — scalar structs)

---

## Motivation

Scalar structs are required by:

- Real `.m` scripts that pass labelled data between functions
- Phase 14 (`try/catch`) — the caught exception object `e` is a struct with
  fields `message` and `identifier`
- `dir()` — returns a struct array of file entries (Phase 13.5, deferred)

---

## 13a — Scalar structs (complete)

### Value type

`Value::Struct(IndexMap<String, Value>)` in `env.rs`. Uses the `indexmap`
crate to preserve insertion order (MATLAB-compatible). Added dependency:

```toml
# crates/ccalc-engine/Cargo.toml
indexmap = "2"
```

### Token

`Token::Dot` emitted in the tokenizer only when `.` is followed by an ASCII
letter or underscore. The existing `DotStar` / `DotSlash` / `DotCaret` /
`DotApostrophe` tokens are unaffected — no ambiguity.

### AST changes

| Node | Description |
|------|-------------|
| `Expr::FieldGet(Box<Expr>, String)` | `s.x` — postfix field read; chained via a loop in `parse_primary`: `s.a.b` → `FieldGet(FieldGet(Var("s"),"a"),"b")` |
| `Stmt::FieldSet(String, Vec<String>, Expr)` | `s.x = rhs` → `("s", ["x"], rhs)`; `s.a.b = rhs` → `("s", ["a","b"], rhs)` |

### Parser

`try_split_field_assign()` — byte-level string scan that detects the pattern
`ident (.ident)+ =` before tokenization. Called first in `parse()`, before
`try_split_cell_assign`.

`parse_primary()` has a postfix loop that handles `Token::Dot` after any
expression to build a `FieldGet` chain.

### Execution — `Stmt::FieldSet`

Implemented in `exec.rs`:

1. Remove the root variable from `Env` (or start with an empty `IndexMap` if
   the variable doesn't exist yet).
2. Call `set_nested(map, path, value)` — a recursive, ownership-by-value
   helper that walks the `Vec<String>` path, creating intermediate structs
   where needed.
3. Re-insert the updated `Value::Struct` into `Env`.
4. Display the struct if not silent.

### Built-ins

| Function | Behaviour |
|----------|-----------|
| `struct('k1',v1,...)` | Constructor; requires an even number of arguments; `struct()` returns empty struct |
| `fieldnames(s)` | Returns `Value::Cell` of `Value::Str` names in insertion order |
| `isfield(s, 'x')` | `Scalar(1.0)` or `Scalar(0.0)` |
| `rmfield(s, 'x')` | Copy of struct without the named field; error if absent |
| `isstruct(v)` | `Scalar(1.0)` if `Value::Struct`, else `Scalar(0.0)` |

`struct()` and the other struct built-ins skip the `ans`-injection logic
(zero-argument built-in calls normally inject `ans` as the first argument —
this is suppressed for struct/cell utilities via the `no_ans_inject` list in
`eval.rs`).

### Display

```
s =

  struct with fields:

    x: 1
    y: [1×3 double]
    inner: [1×1 struct]
```

- Inline format (`format_value`): `[1×1 struct]`
- Full format (`format_value_full`): the `struct with fields:` block above
- Nested struct fields: always shown inline as `[1×1 struct]`

### Exhaustive match coverage

`Value::Struct(_)` was added to every exhaustive `match` arm across
`eval.rs`, `exec.rs`, `repl.rs`, and `repl_tests.rs`:

- Arithmetic, comparison, unary ops → error
- `size`/`length`/`numel` → returns 1 / `[1 1]` (treats struct as 1×1)
- `eval_index` with `()` → helpful error message
- `is_truthy` → `true`
- Display arms in test harness → delegates to `format_value_full`

### Tests

19 regression tests in `crates/ccalc-engine/src/parser_tests.rs`:

| Test | What it checks |
|------|---------------|
| `test_struct_field_assign_basic` | `s.x = 42` stores scalar |
| `test_struct_field_read` | `s.x = 7; ans = s.x` returns 7 |
| `test_struct_multiple_fields` | Three fields stored correctly |
| `test_struct_field_overwrite` | Re-assigning a field updates it |
| `test_struct_nested_assign` | `s.a.b = 5` creates nested struct |
| `test_struct_nested_read` | `s.a.b = 10; ans = s.a.b` returns 10 |
| `test_struct_constructor_basic` | `struct('x',1,'y',2)` |
| `test_struct_constructor_empty` | `struct()` returns empty struct |
| `test_struct_fieldnames` | Returns correct Cell of Str |
| `test_struct_isfield_true/false` | Both cases |
| `test_struct_rmfield` | Field removed, others intact |
| `test_struct_isstruct_true/false` | Both cases |
| `test_struct_field_missing_error` | Access of absent field → error |
| `test_struct_field_on_non_struct_error` | `.field` on non-struct → error |
| `test_struct_constructor_odd_args_error` | `struct('x')` → error |
| `test_struct_rmfield_missing_error` | `rmfield(s,'z')` → error |
| `test_struct_field_insertion_order` | IndexMap preserves order |

---

## 13b — Struct arrays (deferred → Phase 13.5)

`s(i).field` — indexing into a vector of structs. Required for `e.stack` in
`catch e` (Phase 14). Design decision deferred.

## 13c — Dynamic field access (deferred → §3)

```matlab
fname = 'x';
v = s.(fname);    % read via string variable
s.(fname) = 1;   % write via string variable
```
