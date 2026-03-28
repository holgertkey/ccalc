# Memory Cells

ccalc has nine named memory cells: **m1** through **m9**.

## Storing

| Syntax | Action |
|---|---|
| `m[1-9]` | Store current accumulator into cell |
| `expr m[1-9]` | Evaluate expression, store result into cell |

```
[ 0 ]: 100
[ 100 ]: m1           # m1 = 100
[ 100 ]: (1 + 1) * 3 m2   # m2 = 6
```

## Recalling

Use `m[1-9]` inside any expression — it is replaced with the cell's value:

```
[ 0 ]: m1 + m2        →  106    (100 + 6)
[ 0 ]: sqrt(m1)       →   10
```

## Compound assignment

Combine a store with an arithmetic operation in one step.
The result is written back to the cell **and** becomes the new accumulator.

| Syntax | Operation |
|---|---|
| `expr m[1-9]+` | cell += expr |
| `expr m[1-9]-` | cell -= expr |
| `expr m[1-9]*` | cell *= expr |
| `expr m[1-9]/` | cell /= expr |
| `expr m[1-9]%` | cell %= expr |
| `expr m[1-9]^` | cell ^= expr |

```
[ 0 ]: 100 m1         # m1 = 100
[ 100 ]: 2 m1*        # m1 = 200; accumulator = 200
[ 200 ]: 50 m1-       # m1 = 150; accumulator = 150
[ 150 ]: 3 m1/        # m1 = 50;  accumulator = 50
```

## Copying a cell

`expr m[1-9]` stores the expression result into the cell.
To copy m1 into m2, use the cell value as the expression:

```
[ 0 ]: m1 m2          # m2 = value of m1
```

## View and clear

| Command | Action |
|---|---|
| `m` | Show all non-zero cells |
| `mc` | Clear all cells |
| `mc[1-9]` | Clear a specific cell |

```
[ 50 ]: m
m1: 50
m2: 6
[ 50 ]: mc1
[ 50 ]: m
m2: 6
[ 50 ]: mc
[ 50 ]: m
(nothing)
```

## Persistence

| Command | Action |
|---|---|
| `ms` | Save cells to `~/.config/ccalc/memory.toml` |
| `ml` | Load cells from file (clears current cells first) |

The file is plain text in TOML-like format:

```
m1 = 100
m3 = 3.14
m9 = -7.5
```

Cells that are zero are not written to the file.
