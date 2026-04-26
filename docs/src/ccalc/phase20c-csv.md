# Phase 20c — CSV Improvements

Introduced in **v0.24.0+001**.

Phase 20c adds `readmatrix`, `readtable`, and `writetable` built-ins, extending
the existing `dlmread`/`dlmwrite` infrastructure with header handling, mixed-type
columns, and RFC 4180 quoting.

---

## Built-ins

### `readmatrix(path)` / `readmatrix(path, 'Delimiter', d)`

Reads a delimiter-separated numeric file and returns a `Value::Matrix`.

- Delimiter auto-detection: comma (CSV-aware, respects quoted fields) → tab → whitespace.
- First row heuristic: if any non-empty field fails `f64::parse`, the row is skipped as a header.
  A purely numeric first row is treated as data.
- Empty cells → `f64::NAN` (differs from `dlmread`'s `0.0`).
- Errors if any data cell is non-numeric.

### `readtable(path)` / `readtable(path, 'Delimiter', d)`

Reads a CSV file with a mandatory header row and returns a `Value::Struct` of columns.

**Column type rules:**

| Condition | ccalc value |
|-----------|-------------|
| All cells parseable as `f64` (empty → `NaN`) | `Matrix` N×1 column vector |
| Any non-numeric cell | `Cell` of `Value::Str` |

Header names are sanitized: non-alphanumeric runs collapse to `_`, leading digits
get an `x` prefix, empty headers become `x{N}`. Duplicate names get `_1`, `_2`, …
suffixes.

Returns an empty `Struct` for an empty file; returns a struct of zero-row `Matrix`
columns for a header-only file.

### `writetable(T, path)` / `writetable(T, path, 'Delimiter', d)`

Writes a `Value::Struct` table to a CSV file with a header row.

- Accepted column types: `Matrix` (N×1), `Cell`, `Scalar`, `Str`/`StringObj`.
- Non-column matrices (M×N where N≠1) are rejected.
- All columns must have the same row count.
- RFC 4180 quoting: cells containing the delimiter, `"`, or `\n` are wrapped
  in `"..."` with internal `"` doubled.
- Returns `Value::Void`.

---

## Implementation

All new code lives in `crates/ccalc-engine/src/eval.rs` under a
`// --- CSV read/write helpers ---` comment block after `dlmwrite_impl`.

### Helper functions

| Function | Purpose |
|----------|---------|
| `auto_detect_delim(lines)` | CSV-aware comma check, then tab, then `None` |
| `split_csv_row(line, delim)` | RFC 4180 field split with `""` escape support |
| `split_csv_row_opt(line, delim)` | Wraps `split_csv_row`; `None` → `split_whitespace` |
| `row_is_header(fields)` | `true` if any non-empty field is non-numeric |
| `sanitize_header(s, col)` | Converts raw header to identifier-like name |
| `deduplicate_headers(headers)` | Appends `_N` suffixes to duplicate names |
| `parse_delimiter_opt(fn, args, start)` | Parses optional `('Delimiter', d)` arg pair |
| `readmatrix_impl(path, delim)` | Core `readmatrix` logic |
| `readtable_impl(path, delim)` | Core `readtable` logic |
| `csv_quote_cell(s, delim)` | RFC 4180 quoting |
| `col_nrows(v)` | Row count for a struct column value |
| `col_cell_str(v, row, delim)` | Formatted CSV cell for one row |
| `writetable_impl(tbl, path, delim)` | Core `writetable` logic |

Match arms added in `call_builtin` near the existing `dlmread`/`dlmwrite` arms.
All three names added to `builtin_names()` (alphabetical order).

---

## Tests

15 tests in `eval_tests.rs` under `mod csv_tests`:

```bash
cargo test csv_tests
```

Coverage: numeric matrix, header skip, numeric-first-row-not-header, explicit tab
delimiter, empty cells → NaN, empty file, numeric columns, mixed columns, header-only
file, quoted field with embedded comma, empty file (readtable), basic write, quoting,
roundtrip (writetable → readtable), wrong column type error.
