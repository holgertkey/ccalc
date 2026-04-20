# Functions & Constants

## One-argument functions

| Function   | Description                                              | Example                           |
|------------|----------------------------------------------------------|-----------------------------------|
| `sqrt(x)`  | Square root of `x`                                       | `sqrt(144)` → `12`                |
| `abs(x)`   | Absolute value of `x`                                    | `abs(-7)` → `7`                   |
| `floor(x)` | Largest integer ≤ `x` (round toward −∞)                 | `floor(2.9)` → `2`                |
| `ceil(x)`  | Smallest integer ≥ `x` (round toward +∞)                | `ceil(2.1)` → `3`                 |
| `round(x)` | Nearest integer; ties round away from zero               | `round(2.5)` → `3`                |
| `sign(x)`  | −1 if `x < 0`, 0 if `x = 0`, 1 if `x > 0`              | `sign(-5)` → `-1`                 |
| `log(x)`   | Natural logarithm of `x`, base *e* (requires `x > 0`)   | `log(e)` → `1`                    |
| `log2(x)`  | Base-2 logarithm of `x` (requires `x > 0`)              | `log2(8)` → `3`                   |
| `log10(x)` | Base-10 logarithm of `x` (requires `x > 0`)             | `log10(1000)` → `3`               |
| `exp(x)`   | *e* raised to the power `x`                              | `exp(1)` → `2.71828…`             |
| `sin(x)`   | Sine of `x`, where `x` is in **radians**                 | `sin(pi/6)` → `0.5`               |
| `cos(x)`   | Cosine of `x`, where `x` is in **radians**               | `cos(0)` → `1`                    |
| `tan(x)`   | Tangent of `x`, where `x` is in **radians**              | `tan(pi/4)` → `1`                 |
| `asin(x)`  | Inverse sine of `x` ∈ [−1, 1]; result in [−π/2, π/2]   | `asin(1) * 180/pi` → `90`         |
| `acos(x)`  | Inverse cosine of `x` ∈ [−1, 1]; result in [0, π]       | `acos(0) * 180/pi` → `90`         |
| `atan(x)`  | Inverse tangent of `x`; result in (−π/2, π/2)           | `atan(1) * 180/pi` → `45`         |

### Notes

**`sqrt(x)`** — `x` must be ≥ 0. Passing a negative value returns `NaN` (no error is raised).

**`floor`, `ceil`, `round`** — All three return a floating-point result, not an integer type.  
`round` uses *round-half-away-from-zero*: `round(0.5) → 1`, `round(-0.5) → -1`.  
Compare: `floor(-2.5) → -3`, `ceil(-2.5) → -2`, `round(-2.5) → -3`.

**`sign(x)`** — Returns `NaN` when `x` is `NaN` (sign of NaN is undefined).

**`log(x)`** — Natural logarithm (base *e*), MATLAB/Octave-compatible. Returns `NaN` for `x < 0` and `-Inf` for `x = 0`. No error is raised.  
**`log2(x)`** — Base-2 logarithm. **`log10(x)`** — Base-10 logarithm.  
For an arbitrary base see `log(x, base)`.

**`sin`, `cos`, `tan`** — Expect `x` in **radians**. To convert from degrees: `deg * pi / 180`.  
`tan(x)` is undefined at `x = π/2 + n·π`; it returns `±Inf` at those points.

**`asin(x)`, `acos(x)`** — Domain is [−1, 1]; values outside return `NaN`.  
To get degrees: multiply the result by `180/pi`.

**`atan(x)`** — Handles all finite inputs; returns a value in the open interval (−π/2, π/2).  
It cannot determine the quadrant because it only sees the ratio `y/x`.
Use `atan2(y, x)` when you need a four-quadrant result.

```
sqrt(144)           →   12
abs(-7)             →    7
floor(2.9)          →    2
ceil(2.1)           →    3
round(2.5)          →    3
sign(-5)            →   -1
log(e)              →    1     (natural log)
log2(8)             →    3
log10(1000)         →    3
exp(log(5))         →    5     (round-trip)
sin(pi / 6)         →    0.5
cos(pi / 3)         →    0.5
tan(pi / 4)         →    1
asin(0.5) * 180/pi  →   30
acos(0.5) * 180/pi  →   60
atan(1)   * 180/pi  →   45
```

## Two-argument functions

| Function        | Description                                                       | Example                        |
|-----------------|-------------------------------------------------------------------|--------------------------------|
| `atan2(y, x)`   | Four-quadrant inverse tangent; result in (−π, π]                  | `atan2(1,1)*180/pi` → `45`     |
| `mod(a, b)`     | Remainder of `a ÷ b`; result has the **sign of `b`**              | `mod(370, 360)` → `10`         |
| `rem(a, b)`     | Remainder of `a ÷ b`; result has the **sign of `a`**              | `rem(-1, 3)` → `-1`            |
| `max(a, b)`     | Larger of two scalar values                                       | `max(3, 7)` → `7`              |
| `min(a, b)`     | Smaller of two scalar values                                      | `min(3, 7)` → `3`              |
| `hypot(a, b)`   | Euclidean distance √(a²+b²), numerically stable                   | `hypot(3, 4)` → `5`            |
| `log(x, base)`  | Logarithm of `x` to an arbitrary `base` (both must be > 0)       | `log(8, 2)` → `3`              |

### Notes

**`atan2(y, x)`** — First argument is `y` (numerator), second is `x` (denominator).  
Returns a value in the range (−π, π], correctly determining the quadrant from the signs of both arguments.  
`atan2(0, -1) * 180/pi → 180`, whereas `atan(-0/-1) * 180/pi → 0`.

**`mod(a, b)` vs `rem(a, b)`** — Both compute the remainder after division, but differ in sign
when the operands have opposite signs:

```
mod(-1, 3)   →   2    (result has the sign of 3, in range [0, 3))
rem(-1, 3)   →  -1    (result has the sign of -1)
```

`mod` guarantees the result is in `[0, b)` for positive `b`, making it useful for angle wrapping and modular arithmetic.
`rem` follows IEEE 754 remainder convention.
`b = 0` produces `NaN`.

**`max(a, b)`, `min(a, b)`** — These two-argument forms work with scalars only.
To find the maximum or minimum element of a vector or matrix, use the one-argument form `max(v)` / `min(v)` (see [Vector & Data Utilities](vectors.md)).

**`hypot(a, b)`** — Computes √(a²+b²) without intermediate overflow or underflow.
Prefer `hypot` over `sqrt(a^2 + b^2)` when the values may be very large or very small.

**`log(x, base)`** — Both `x` and `base` must be positive; `base` must not equal 1.
Negative or zero values return `NaN` or `-Inf` as with the single-argument form.

```
atan2(1, 1) * 180/pi   →   45
atan2(0, -1) * 180/pi  →  180
mod(370, 360)          →   10
mod(-1, 3)             →    2     (result in [0, 3))
rem(-1, 3)             →   -1     (same sign as dividend)
max(3, 7)              →    7
min(3, 7)              →    3
hypot(3, 4)            →    5
hypot(5, 12)           →   13
log(8, 2)              →    3     (log base 2 of 8)
log(100, 10)           →    2     (same as log10(100))
```

### `mod` vs `rem`

Both compute the remainder after division, but differ in sign when the operands
have opposite signs:

```
mod(-1, 3)   →   2    (result has the sign of 3)
rem(-1, 3)   →  -1    (result has the sign of -1)
```

Use `mod` when you want a value always in `[0, b)`, e.g. for angle wrapping.
Use `rem` when you need the IEEE 754 remainder.

## Bitwise functions

All bitwise functions require **non-negative integer** arguments.
They pair naturally with hex (`0xFF`), binary (`0b1010`), and octal (`0o17`)
input literals.

| Function           | Description                                                                          |
|--------------------|--------------------------------------------------------------------------------------|
| `bitand(a, b)`     | Bitwise AND of `a` and `b`                                                           |
| `bitor(a, b)`      | Bitwise OR of `a` and `b`                                                            |
| `bitxor(a, b)`     | Bitwise XOR of `a` and `b`                                                           |
| `bitshift(a, n)`   | Shift `a` left by `n` bits (`n > 0`) or right by `|n|` bits (`n < 0`); returns 0 if `|n| ≥ 64` |
| `bitnot(a)`        | Bitwise NOT of `a` within a **32-bit** window                                        |
| `bitnot(a, bits)`  | Bitwise NOT of `a` within an explicit `bits`-wide window; `bits` ∈ [1, 53]          |

### Notes

**`bitand`, `bitor`, `bitxor`** — Both arguments must be non-negative integers.
Floating-point values are truncated toward zero before the operation.

**`bitshift(a, n)`** — Positive `n` shifts left (multiply by 2ⁿ); negative `n` shifts right (logical, fills with zeros).
Returns 0 when `|n| ≥ 64`. The shift count `n` may be negative; `a` must be non-negative.

**`bitnot(a)`** — Flips all bits within a 32-bit window (Octave `uint32` default).
Result is `2³² − 1 − a` for values that fit in 32 bits.

**`bitnot(a, bits)`** — Flips bits within a window of `bits` width.
`bits` must be in [1, 53] (limited to the integer precision of IEEE 754 doubles).
Result is `2^bits − 1 − a`.

```
bitand(0xFF, 0x0F)      →   15
bitor(0b1010, 0b0101)   →   15
bitxor(0xFF, 0x0F)      →  240     (0xF0)
bitshift(1, 8)          →  256     (1 << 8)
bitshift(256, -4)       →   16     (256 >> 4)
bitnot(5, 8)            →  250     (~5 within 8 bits = 0b11111010)
bitnot(0, 32)           →  4294967295   (0xFFFFFFFF)
```

Combining shifts and masks:

```
bitshift(1, 4) - 1      →   15     (0b00001111 — 4-bit all-ones mask)
bitand(0xDEAD, 0xFF00)  →  56832   (0xDE00 — extract high byte)
```

## Empty-argument shorthand

Calling a function with empty parentheses uses **ans** as the argument:

```
[ 144 ]: sqrt()      →  12     (same as sqrt(144))
[ -7 ]:  abs()       →   7
[ 0 ]:   sin()       →   0
```

## Constants

| Name  | Value                                                     |
|-------|-----------------------------------------------------------|
| `pi`  | 3.14159265358979…                                         |
| `e`   | 2.71828182845904…                                         |
| `nan` / `NaN` | IEEE 754 Not-a-Number — propagates through all arithmetic |
| `inf` / `Inf` | Positive infinity; use `-inf` for negative infinity       |
| `ans` | Result of the last expression                             |

**`nan`** — Not a number. Any arithmetic operation involving `nan` returns `nan`.
`nan == nan` evaluates to 0 (IEEE 754: NaN is never equal to itself).
Use `isnan(x)` to test for NaN (see [Vector & Data Utilities](vectors.md)).

**`inf`** — Positive infinity. `-inf` is negative infinity.
`1 / inf → 0`, `-inf < inf → 1`, `inf + inf → inf`, `inf - inf → nan`.

**`ans`** — Holds the result of the most recent **expression** (assignments do not update it).
`ans` can appear anywhere in an expression.

`nan` and `inf` are parser-level constants and cannot be overwritten by assignment.

```
nan + 5         % → NaN
nan == nan      % → 0   (IEEE 754: NaN is never equal to itself)
1 / inf         % → 0
-inf < inf      % → 1
```

`ans` can appear anywhere in an expression:

```
[ 9 ]: ans * 2 + 1    →  19
[ 9 ]: sqrt(ans)      →   3
```

## Nesting

Functions can be nested freely:

```
sqrt(abs(-16))          →    4
log(exp(1))             →    1
floor(sqrt(10))         →    3
max(hypot(3,4), 6)      →    6
```

## Functions in expressions

```
sqrt(144) + 3           →   15
2 * sin(pi / 6)         →    1
log10(1000) ^ 2         →    9
hypot(3, 4) * 2         →   10
atan2(1, 1) * 180 / pi  →   45
```

See also: [Vector & Data Utilities](vectors.md) for `sum`, `prod`, `mean`, `norm`, `sort`, `find`, and related functions.
