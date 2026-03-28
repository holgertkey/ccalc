# Arithmetic & Operators

## Operators

| Operator | Operation | Example |
|---|---|---|
| `+` | Addition | `3 + 4` → `7` |
| `-` | Subtraction / unary minus | `10 - 4` → `6`, `-5` |
| `*` | Multiplication | `3 * 7` → `21` |
| `/` | Division | `10 / 4` → `2.5` |
| `^` | Exponentiation (right-associative) | `2 ^ 10` → `1024` |
| `%` | Modulo or percentage (context-sensitive) | `17 % 5` → `2` |

## Precedence (high → low)

1. `^` — right-associative: `2 ^ 3 ^ 2` = `2 ^ (3 ^ 2)` = `512`
2. `*`, `/`, `%`, implicit multiplication
3. `+`, `-`

Use parentheses to override: `(2 + 3) * 4` → `20`.

## Partial expressions

An expression starting with an operator uses the accumulator as the left operand:

```
[ 100 ]: * 2
[ 200 ]: ^ 2
[ 40000 ]: % 1000
[ 0 ]:
```

## Percentage operator `%`

`%` is **context-sensitive**:

- Followed by a number or expression → **modulo**: `17 % 5` → `2`
- At end of input → **percentage of accumulator**: `N% = N * (acc / 100)`

```
[ 1500 ]: 20%         →  300    (20% of 1500)
[ 1500 ]: + 20%       →  1800   (add 20% to 1500)
[ 1800 ]: - 10%       →  1620   (subtract 10% from 1800)
```

## Implicit multiplication

A number or closing parenthesis immediately before `(` multiplies:

```
2(3 + 1)      →   8      (same as 2 * (3 + 1))
(2 + 1)(4)    →  12
2(3)(4)       →  24
```

## Unary minus

```
-5
-(3 + 2)      →  -5
--5           →   5
```
