# Phase 22 — Datetime & Duration

Adds UTC datetime and duration types as first-class values.

## New value types

| Variant | Storage | Notes |
|---|---|---|
| `DateTime(f64)` | Unix timestamp (seconds) | `NaN` = NaT |
| `Duration(f64)` | Seconds (fractional) | |
| `DateTimeArray(Vec<f64>)` | Flat timestamp vec | Display as N×1 |
| `DurationArray(Vec<f64>)` | Flat seconds vec | Display as N×1 |

## New module: `ccalc-engine::datetime`

Pure-Rust UTC calendar arithmetic; no external crate.
Uses the Howard Hinnant proleptic Gregorian algorithm.

Key functions: `days_from_civil`, `civil_from_days`, `timestamp_to_civil`,
`civil_to_timestamp`, `parse_iso8601`, `format_datetime`, `format_duration`,
`format_datestr`, `now_timestamp`, `today_timestamp`, `to_datenum`, `from_datenum`.

## Parser change

`NaT` added as a parser-level constant (like `pi`, `nan`) in `parse_primary` →
`Expr::NaT` → `Value::DateTime(f64::NAN)`. Not env-seeded so user cannot overwrite it.

## New builtins

`datetime`, `duration`, `hours`, `minutes`, `seconds`, `days`, `milliseconds`, `years`,
`year`, `month`, `day`, `hour`, `minute`, `second`,
`isdatetime`, `isduration`, `isnat`,
`datestr`, `datevec`, `datenum`, `posixtime`, `diff` (extended).

## Tests

49 tests in `eval_tests.rs::datetime_tests`:
constructors, extractors, predicates, arithmetic, formatting, array operations.
