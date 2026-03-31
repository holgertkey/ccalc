# Functions & Constants

## One-argument functions

| Function   | Description                                          |
|------------|------------------------------------------------------|
| `sqrt(x)`  | Square root                                          |
| `abs(x)`   | Absolute value                                       |
| `floor(x)` | Round down to integer                                |
| `ceil(x)`  | Round up to integer                                  |
| `round(x)` | Round to nearest integer                             |
| `sign(x)`  | Sign: −1, 0, or 1                                    |
| `log(x)`   | Base-10 logarithm                                    |
| `ln(x)`    | Natural logarithm (base *e*)                         |
| `exp(x)`   | *e* raised to the power *x*                          |
| `sin(x)`   | Sine (radians)                                       |
| `cos(x)`   | Cosine (radians)                                     |
| `tan(x)`   | Tangent (radians)                                    |
| `asin(x)`  | Inverse sine, result in radians                      |
| `acos(x)`  | Inverse cosine, result in radians                    |
| `atan(x)`  | Inverse tangent, result in radians                   |

## Two-argument functions

| Function        | Description                                              |
|-----------------|----------------------------------------------------------|
| `atan2(y, x)`   | Four-quadrant inverse tangent, result in radians         |
| `mod(a, b)`     | Remainder, sign follows the divisor (Octave convention)  |
| `rem(a, b)`     | Remainder, sign follows the dividend                     |
| `max(a, b)`     | Larger of two values                                     |
| `min(a, b)`     | Smaller of two values                                    |
| `hypot(a, b)`   | Euclidean distance √(a²+b²), numerically stable          |
| `log(x, base)`  | Logarithm of *x* to an arbitrary *base*                  |

### `mod` vs `rem`

Both compute the remainder after division, but differ in sign when the operands
have opposite signs:

```
mod(-1, 3)   →   2    (result has the sign of 3)
rem(-1, 3)   →  -1    (result has the sign of -1)
```

Use `mod` when you want a value always in `[0, b)`, e.g. for angle wrapping.
Use `rem` when you need the IEEE 754 remainder.

## Empty-argument shorthand

Calling a function with empty parentheses uses **ans** as the argument:

```
[ 144 ]: sqrt()      →  12     (same as sqrt(144))
[ -7 ]:  abs()       →   7
[ 0 ]:   sin()       →   0
```

## Constants

| Name  | Value                         |
|-------|-------------------------------|
| `pi`  | 3.14159265358979…             |
| `e`   | 2.71828182845904…             |
| `ans` | Result of last expression     |

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
