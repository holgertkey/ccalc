# Phase 13.5 — Struct Arrays

**Version:** 0.19.0+001
**Status:** Complete

---

## Motivation

`s(i).field` syntax is required by:

- Real `.m` scripts that work with collections of labelled records (e.g. measurement
  series, roster data, inventory ledgers)
- Phase 14 (`try/catch`) — the `stack` field of a caught exception object `e` is a
  struct array of call frames
- `dir()` — returns a struct array of directory entries (planned)

---

## Value type

`Value::StructArray(Vec<IndexMap<String, Value>>)` in `env.rs`.

A new variant separate from scalar `Value::Struct`. Each `Vec` element is one
struct (an `IndexMap` mapping field name → `Value`). Using a separate variant
keeps `Value::Struct` paths unchanged and makes pattern matching unambiguous.

---

## AST changes

| Node | Description |
|------|-------------|
| `Stmt::StructArrayFieldSet(String, Expr, Vec<String>, Expr)` | `s(i).f1.f2 = rhs` — base name, index expression, field path, right-hand side |

---

## Parser

`try_split_struct_array_field_assign(input)` — byte-level scan detecting the
pattern `ident(...)(.ident)+ =` before tokenization. It tracks bracket depth
to correctly skip any expression inside `(...)`. Called first in `parse()`,
before `try_split_field_assign`, so that `s(1).x = val` is never mistakenly
parsed as a plain field assignment.

`parse_primary()` handles `s(i).field` reads via existing `FieldGet` postfix
loop: `eval_index` is called for `s(i)`, returning a `Value::Struct`, and
the `.field` suffix is then applied normally.

---

## Execution

### Write — `Stmt::StructArrayFieldSet`

Implemented in `exec.rs`:

1. Evaluate the index expression; convert to 1-based `usize`.
2. Remove the root variable from `Env`:
   - `Value::StructArray(v)` → use as-is.
   - `Value::Struct(m)` → promote to `vec![m]` (1-element array).
   - Missing → start with empty `Vec`.
3. Auto-grow: append empty `IndexMap`s until `arr.len() >= idx`.
4. Call existing `set_nested(elem, path, rhs)` on `arr[idx - 1]`.
5. Re-insert `Value::StructArray(arr)` into `Env`.

### Read — `eval_index` on `StructArray`

- Single index `s(i)` → `Value::Struct` (clone of element).
- Range or `:` → `Value::StructArray` (cloned sub-array).

### Field collection — `Expr::FieldGet` on `StructArray`

`s.field` with no index: iterates all elements, collects the named field:

- All elements yield `Value::Scalar` → `Value::Matrix` 1×N row vector.
- Any non-scalar element → `Value::Cell`.

---

## Built-ins extended for `StructArray`

| Built-in | Behaviour on `StructArray` |
|----------|---------------------------|
| `isstruct(s)` | Returns `1.0` (same as scalar struct) |
| `fieldnames(s)` | Uses field names of the first element |
| `isfield(s, 'x')` | Tests first element's field map |
| `rmfield(s, 'x')` | Removes field from every element; returns new `StructArray` |
| `numel(s)` | Returns element count as `Scalar` |
| `size(s)` | Returns `[1, N]` as 1×2 matrix |
| `size(s, 1)` | Returns `1` |
| `size(s, 2)` | Returns `N` |
| `length(s)` | Returns `max(1, N)` |

---

## Display

- Inline (`format_value`): `[1×N struct]`
- Full (`format_value_full`):
  - N > 1: field names list, e.g. `1×3 struct array with fields: x  y`
  - N = 1: full `struct with fields:` block (same as scalar struct)

---

## Tests

8 regression tests added in `crates/ccalc-engine/src/parser_tests.rs`:

| Test | What it checks |
|------|---------------|
| `test_struct_array_create_and_read` | Basic `s(1).x` / `s(2).x` round-trip |
| `test_struct_array_numel` | `numel` returns element count |
| `test_struct_array_isstruct` | `isstruct` returns 1 |
| `test_struct_array_field_collection_scalar` | `s.x` → `Matrix` when all scalar |
| `test_struct_array_auto_grow` | `s(3).x` grows array past current length |
| `test_struct_array_nested_field` | Nested path `s(1).reading.temp = 22.5` |
| `test_struct_array_fieldnames` | `fieldnames` on struct array |
| `test_struct_array_isfield` | `isfield` on struct array |

---

## Example

See `examples/struct_arrays.calc` for a comprehensive 8-section example
covering creation, element access, field collection, loop building,
`fieldnames`/`isfield`, string field collection into a cell, nested fields,
and a practical inventory-ledger calculation.
