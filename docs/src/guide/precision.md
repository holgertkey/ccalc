# Number Display Format

The `format` command controls how numbers are displayed in the REPL and
script/pipe output. It does not affect computation — all arithmetic is done in
`f64` (IEEE 754 double precision).

## Commands

| Command          | Description                                        |
|------------------|----------------------------------------------------|
| `format`         | Reset to `short` (5 significant digits)            |
| `format short`   | 5 significant digits, auto fixed/scientific        |
| `format long`    | 15 significant digits, auto fixed/scientific       |
| `format shortE`  | Always scientific, 4 decimal places                |
| `format longE`   | Always scientific, 14 decimal places               |
| `format shortG`  | Same as `short` (MATLAB `shortG` alias)            |
| `format longG`   | Same as `long` (MATLAB `longG` alias)              |
| `format bank`    | Fixed 2 decimal places (currency)                  |
| `format rat`     | Rational approximation `p/q`                       |
| `format hex`     | IEEE 754 double-precision bit pattern (16 hex digits) |
| `format +`       | Sign only: `+` positive, `-` negative, space for 0 |
| `format compact` | Suppress blank lines between outputs               |
| `format loose`   | Add blank line after every output (default)        |
| `format N`       | N decimal places (e.g. `format 4`)                 |

## Examples

```
>> format short
>> pi
3.1416

>> format long
>> pi
3.14159265358979

>> format shortE
>> pi
3.1416e+00

>> format bank
>> 1/3
0.33

>> format rat
>> pi
355/113

>> format hex
>> 1.0
3FF0000000000000

>> format +
>> [-2 0 5]
- +

>> format 4
>> 1/3
0.3333
```

## Scope

`format` affects:
- `disp()` output
- Variable assignment display (`x = 3.1416`)
- The REPL prompt value

`format` does **not** affect `fprintf` / `sprintf` — those functions use
their own per-call format specifiers (e.g. `%f`, `%e`, `%.3f`).

## Automatic scientific notation

In `short` and `long` modes, numbers switch to scientific notation when:
- The exponent is less than −3 (e.g. `0.001` → `1e-03`)
- The exponent is ≥ the number of significant digits

## Persistent default

The default precision (used by `format N` and the startup default) is set in
[`config.toml`](./configuration.md):

```toml
[display]
precision = 10
```

## Note: `format hex` vs `hex`

These are different commands:

- `format hex` — shows the IEEE 754 raw bit pattern of any floating-point
  number as 16 uppercase hex digits (e.g. `400921FB54442D18` for `pi`).
- `hex` — switches the display base to hexadecimal for integer values
  (e.g. `0xFF` → `255` shown as `0xFF`).
