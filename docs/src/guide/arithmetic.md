# Arithmetic & Operators

## Scalar operators

| Operator | Operation | Example |
|---|---|---|
| `+` | Addition | `3 + 4` → `7` |
| `-` | Subtraction / unary minus | `10 - 4` → `6`, `-5` |
| `*` | Multiplication | `3 * 7` → `21` |
| `/` | Division | `10 / 4` → `2.5` |
| `^` | Exponentiation (right-associative) | `2 ^ 10` → `1024` |

For modulo use the `mod(a, b)` function. `%` is a comment character.

## Comparison operators

Return `1.0` (true) or `0.0` (false). Work element-wise on matrices.

| Operator | Meaning          |
|----------|------------------|
| `==`     | Equal            |
| `~=`     | Not equal        |
| `<`      | Less than        |
| `>`      | Greater than     |
| `<=`     | Less or equal    |
| `>=`     | Greater or equal |

## Logical operators

| Operator | Meaning         |
|----------|-----------------|
| `~expr`  | Logical NOT     |
| `&&`     | Logical AND     |
| `\|\|`   | Logical OR      |

See [Comparison & Logical Operators](./logic.md) for full details.

## Precedence (high → low)

1. postfix `'` — transpose
2. `^`, `.^` — right-associative
3. unary `-`, `~` — negation, logical NOT
4. `*`, `/`, `.*`, `./`, `.^`, implicit multiplication
5. `+`, `-`
6. `:` — range
7. `==`, `~=`, `<`, `>`, `<=`, `>=` — comparison (non-associative)
8. `&&` — logical AND
9. `||` — logical OR (lowest)

Use parentheses to override: `(2 + 3) * 4` → `20`.

## Partial expressions

An expression starting with an operator uses `ans` as the left operand:

```
[ 100 ]: / 4
[ 25 ]: ^ 2
[ 625 ]:
```

## Implicit multiplication

A number, variable, or closing parenthesis immediately before `(` multiplies:

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

## Matrix operators

When one or both operands are matrices, the same operators apply with
element-wise or broadcast semantics:

| Expression | Semantics |
|---|---|
| `scalar + matrix` | Add scalar to every element |
| `matrix + matrix` | Element-wise (shapes must match) |
| `scalar * matrix` | Scale every element |
| `matrix / scalar` | Divide every element |
| `matrix ^ scalar` | Raise every element to the power |

See [Matrices](./matrices.md) for full details.
