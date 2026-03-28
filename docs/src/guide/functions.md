# Functions & Constants

## Built-in functions

| Function | Description |
|---|---|
| `sqrt(x)` | Square root |
| `abs(x)` | Absolute value |
| `floor(x)` | Round down to integer |
| `ceil(x)` | Round up to integer |
| `round(x)` | Round to nearest integer |
| `log(x)` | Base-10 logarithm |
| `ln(x)` | Natural logarithm (base *e*) |
| `exp(x)` | *e* raised to the power *x* |
| `sin(x)` | Sine (radians) |
| `cos(x)` | Cosine (radians) |
| `tan(x)` | Tangent (radians) |

## Empty-argument shorthand

Calling a function with empty parentheses uses **ans** as the argument:

```
[ 144 ]: sqrt()      →  12     (same as sqrt(144))
[ -7 ]:  abs()       →   7
[ 0 ]:   sin()       →   0
```

## Constants

| Name | Value |
|---|---|
| `pi` | 3.14159265358979… |
| `e` | 2.71828182845904… |
| `ans` | Result of last expression |

`ans` can appear anywhere in an expression:

```
[ 9 ]: ans * 2 + 1    →  19
[ 9 ]: sqrt(ans)      →   3
```

## Nesting

Functions can be nested freely:

```
sqrt(abs(-16))      →   4
ln(exp(1))          →   1
floor(sqrt(10))     →   3
```

## Functions in expressions

```
sqrt(144) + 3       →  15
2 * sin(pi / 6)     →   1
log(1000) ^ 2       →   9
```
