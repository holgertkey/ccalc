# Functions & Constants

## One-argument functions

| Function   | Description                      | Example                           |
|------------|----------------------------------|-----------------------------------|
| `sqrt(x)`  | Square root                      | `sqrt(144)` → `12`                |
| `abs(x)`   | Absolute value                   | `abs(-7)` → `7`                   |
| `floor(x)` | Round down to integer            | `floor(2.9)` → `2`                |
| `ceil(x)`  | Round up to integer              | `ceil(2.1)` → `3`                 |
| `round(x)` | Round to nearest integer         | `round(2.5)` → `3`                |
| `sign(x)`  | Sign: −1, 0, or 1                | `sign(-5)` → `-1`                 |
| `log(x)`   | Base-10 logarithm                | `log(1000)` → `3`                 |
| `ln(x)`    | Natural logarithm (base *e*)     | `ln(e)` → `1`                     |
| `exp(x)`   | *e* raised to the power *x*      | `exp(1)` → `2.71828…`             |
| `sin(x)`   | Sine (radians)                   | `sin(pi/6)` → `0.5`               |
| `cos(x)`   | Cosine (radians)                 | `cos(0)` → `1`                    |
| `tan(x)`   | Tangent (radians)                | `tan(pi/4)` → `1`                 |
| `asin(x)`  | Inverse sine, result in radians  | `asin(1) * 180/pi` → `90`         |
| `acos(x)`  | Inverse cosine, result in radians| `acos(0) * 180/pi` → `90`         |
| `atan(x)`  | Inverse tangent, result in radians| `atan(1) * 180/pi` → `45`        |

```
sqrt(144)           →   12
abs(-7)             →    7
floor(2.9)          →    2
ceil(2.1)           →    3
round(2.5)          →    3
sign(-5)            →   -1
log(1000)           →    3
ln(e)               →    1
exp(ln(5))          →    5     (round-trip)
sin(pi / 6)         →    0.5
cos(pi / 3)         →    0.5
tan(pi / 4)         →    1
asin(0.5) * 180/pi  →   30
acos(0.5) * 180/pi  →   60
atan(1)   * 180/pi  →   45
```

## Two-argument functions

| Function        | Description                                              | Example                        |
|-----------------|----------------------------------------------------------|--------------------------------|
| `atan2(y, x)`   | Four-quadrant inverse tangent, result in radians         | `atan2(1,1)*180/pi` → `45`     |
| `mod(a, b)`     | Remainder, sign follows the divisor (Octave convention)  | `mod(370, 360)` → `10`         |
| `rem(a, b)`     | Remainder, sign follows the dividend                     | `rem(-1, 3)` → `-1`            |
| `max(a, b)`     | Larger of two values                                     | `max(3, 7)` → `7`              |
| `min(a, b)`     | Smaller of two values                                    | `min(3, 7)` → `3`              |
| `hypot(a, b)`   | Euclidean distance √(a²+b²), numerically stable          | `hypot(3, 4)` → `5`            |
| `log(x, base)`  | Logarithm of *x* to an arbitrary *base*                  | `log(8, 2)` → `3`              |

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
log(100, 10)           →    2     (same as log(100))
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

| Function          | Description |
|-------------------|-------------|
| `bitand(a, b)`    | Bitwise AND |
| `bitor(a, b)`     | Bitwise OR  |
| `bitxor(a, b)`    | Bitwise XOR |
| `bitshift(a, n)`  | Left shift when `n > 0`; logical right shift when `n < 0`; returns 0 if `|n| ≥ 64` |
| `bitnot(a)`       | Bitwise NOT within a 32-bit window (Octave `uint32` default) |
| `bitnot(a, bits)` | Bitwise NOT within an explicit `bits`-wide window (`bits` in [1, 53]) |

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

| Name  | Value                                             |
|-------|---------------------------------------------------|
| `pi`  | 3.14159265358979…                                 |
| `e`   | 2.71828182845904…                                 |
| `nan` | IEEE 754 Not-a-Number — propagates through arithmetic |
| `inf` | Positive infinity (`-inf` for negative infinity)  |
| `ans` | Result of last expression                         |

`nan` and `inf` work exactly like numeric literals:

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
ln(exp(1))              →    1
floor(sqrt(10))         →    3
max(hypot(3,4), 6)      →    6
```

## Functions in expressions

```
sqrt(144) + 3           →   15
2 * sin(pi / 6)         →    1
log(1000) ^ 2           →    9
hypot(3, 4) * 2         →   10
atan2(1, 1) * 180 / pi  →   45
```
