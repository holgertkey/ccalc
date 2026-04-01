# Matrices

ccalc supports matrix literals using Octave/MATLAB bracket syntax.

## Creating matrices

Separate elements with spaces or commas; separate rows with semicolons:

```
[1 2 3]          % row vector  (1×3)
[1; 2; 3]        % column vector  (3×1)
[1 2; 3 4]       % 2×2 matrix
[1, 2, 3]        % commas work too
```

Elements can be arbitrary expressions:

```
[sqrt(4), 2^3, mod(10,3)]     % [2, 8, 1]
[pi/2, pi; -pi, 0]
```

## Assignment

```
[ 0 ]: A = [1 2; 3 4]
A =
   1   2
   3   4

[ [2×2] ]: B = [5 6; 7 8]
B =
   5   6
   7   8
```

Assignment does not update `ans`. The prompt shows the matrix size.

## Arithmetic

### Scalar operations

All four arithmetic operators apply element-wise between a scalar and a matrix:

```
2 * A             % multiply every element by 2
A / 10            % divide every element by 10
A + 1             % add 1 to every element
A ^ 2             % raise every element to the power 2
```

### Matrix addition and subtraction

`+` and `-` between two matrices of the same size are element-wise:

```
A + B
A - B
```

Size must match; otherwise you get an error:

```
[1 2] + [1 2 3]   % Error: Matrix size mismatch for '+'
```

### What is not yet supported

| Operation | Phase |
|---|---|
| Matrix multiplication `A * B` | Phase 4 |
| Transpose `A'` | Phase 4 |
| Element-wise `.*` `./` `.^` | Phase 4 |
| Indexing `A(1,1)` | Phase 6 |
| Range `1:5` | Phase 5 |

## Display

Matrices are displayed with right-aligned columns:

```
ans =
   1    2    3
   4    5    6
   7    8    9
```

The REPL prompt shows the size of the current `ans` when it is a matrix:

```
[ [3×3] ]: 
```

## `who` and workspace

`who` shows matrix dimensions:

```
A = [2×2 double]
x = 3.14
```

`ws` (workspace save) saves only scalar variables. Matrices are not persisted.

## Semicolon inside matrix literals

The `;` inside `[...]` is always a row separator, never a statement separator:

```
A = [1 2; 3 4];   % the ; after ] suppresses output; the ; inside is part of the matrix
```
