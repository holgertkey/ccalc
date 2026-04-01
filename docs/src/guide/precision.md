# Precision

The **precision** setting controls how many decimal places are shown in the
output. It does not affect computation — all arithmetic is done in `f64`
(IEEE 754 double precision).

## Commands

| Command | Action |
|---|---|
| `p` | Show current precision (default: 10) |
| `p<N>` | Set precision to N decimal places (0–15) |

```
[ 0 ]: 1 / 3
[ 0.3333333333 ]: p4
[ 0.3333 ]: 1 / 3
[ 0.3333 ]: p0
[ 0 ]: 1 / 3
[ 0 ]: p10
[ 0 ]: 1 / 3
[ 0.3333333333 ]:
```

## Automatic formatting rules

Regardless of precision, the formatter applies these rules:

- **Integer results** are always shown without a decimal point: `42`, not `42.0`
- **Trailing zeros** are trimmed: `3.14` not `3.1400000000`
- **Very large numbers** (≥ 10¹⁵) switch to scientific notation: `1.5e15`
- **Very small numbers** (< 10⁻⁹) switch to scientific notation: `1.23e-12`

## Non-decimal bases

Precision has no effect when the display base is hex, bin, or oct — those modes
always show the rounded integer value.

## Persistent default

To change the default precision across all sessions, set it in
[`config.toml`](./configuration.md):

```toml
[display]
precision = 4
```
