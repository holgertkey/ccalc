# Comparison & Logical Operators

## Comparison operators

Comparison operators evaluate to `1` (true) or `0` (false).
They work on scalars and element-wise on matrices.

| Operator | Meaning          |
|----------|------------------|
| `==`     | Equal            |
| `~=`     | Not equal        |
| `<`      | Less than        |
| `>`      | Greater than     |
| `<=`     | Less or equal    |
| `>=`     | Greater or equal |

```
3 > 2        % 1
3 == 4       % 0
5 ~= 3       % 1
4 <= 4       % 1
```

Comparison has lower precedence than arithmetic — operands are fully
evaluated before the comparison:

```
1 + 1 == 2   % 1   (1+1 = 2, then 2 == 2)
2 * 3 > 5    % 1   (2*3 = 6, then 6 > 5)
```

## Logical NOT — `~`

`~expr` negates a truth value:

- `0` → `1`
- any non-zero → `0`

```
~0           % 1
~1           % 0
~(3 == 3)    % 0
~(3 ~= 3)    % 1
```

## Short-circuit AND and OR — `&&`, `||`

`&&` returns `1` when both operands are non-zero.
`||` returns `1` when at least one operand is non-zero.
`&&` binds more tightly than `||`.
Both operators **short-circuit** and are intended for scalar conditions.

```
1 && 1       % 1
1 && 0       % 0
0 || 1       % 1
0 || 0       % 0

1 || 0 && 0  % 1    (1 || (0 && 0))
```

### Combining conditions

```
x = 2.7;
x >= 0 && x <= 3.3    % 1  — in-range check
x < 0  || x > 3.3    % 0  — out-of-range flag

% Negate the condition:
~(x >= 0 && x <= 3.3) % 0  — fault flag (0 = OK)
```

## Element-wise logical operators — `&`, `|`, `xor`, `not`

`&` and `|` are **element-wise** operators — they work on matrices, always
evaluate both sides (no short-circuit), and return a `0`/`1` matrix:

```
a = [1 0 1 0];
b = [1 1 0 0];

a & b                  % [1 0 0 0]   element-wise AND
a | b                  % [1 1 1 0]   element-wise OR
xor(a, b)              % [0 1 1 0]   element-wise XOR

not(a)                 % [0 1 0 1]   element-wise NOT (alias for ~)
```

Use `&`/`|` for matrix logical masks; use `&&`/`||` for scalar conditions in `if`.

### Logical mask pattern

```
v = [3, -1, 8, 0, 5, -2, 7];

mask = v > 0 & v < 6   % [1 0 0 0 1 0 0]
```

## Element-wise on matrices (comparison)

When one or both operands are matrices, all comparison operators
apply element-wise and return a `0`/`1` matrix of the same size:

```
v = [1 2 3 4 5];

v > 3              % [0 0 0 1 1]
v <= 3             % [1 1 1 0 0]
v == 3             % [0 0 1 0 0]
~(v > 3)           % [1 1 1 0 0]
```

Scalar–matrix comparison broadcasts the scalar to every element:

```
3 < v              % [0 0 0 1 1]
v >= 3             % [0 0 1 1 1]
```

## Soft masking

Because masks are `0`/`1` matrices, multiplying a matrix by its mask
zeroes out the elements that failed the condition — a pattern often
called *soft masking* or *logical selection*:

```
v = [1 2 3 4 5];

v .* (v > 3)             % [0 0 0 4 5]   keep elements > 3

% Keep elements in [2, 4]:
lo = v >= 2;
hi = v <= 4;
v .* (lo .* hi)          % [0 2 3 4 0]
```

`lo .* hi` works as element-wise AND because the values are already `0`/`1`.

## Precedence

From lowest to highest priority:

```
||          logical OR  (short-circuit)
&&          logical AND (short-circuit)
|           element-wise OR
&           element-wise AND
== ~= < > <= >=   comparison (non-associative)
:           range
+ -         additive
* / .* ./   multiplicative
^ .^ **     power (right-associative)
unary + - ~ negation / logical NOT
postfix ' .' transpose / plain transpose
```

## REPL session

```
[ 0 ]: 3 > 2
[ 1 ]:
[ 0 ]: 5 ~= 5
[ 0 ]:
[ 0 ]: 2 > 1 && 10 > 5
[ 1 ]:
[ 0 ]: v = [10 20 30 40 50];
[ 0 ]: v > 25
ans =
   0   0   0   1   1
[ [1×5] ]: v .* (v > 25)
ans =
    0    0    0   40   50
```

## See also

- `help logic` — REPL reference with examples
- `ccalc examples/logic.calc` — ADC validation and resistor tolerance demo
- [Matrices](./matrices.md) — element-wise operators
