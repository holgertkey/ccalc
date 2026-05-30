# Matrices

ccalc supports matrix literals using Octave/MATLAB bracket syntax.

## Creating matrices

Separate elements with spaces or commas; separate rows with `;` or a bare newline:

```
[1 2 3]          % row vector  (1×3)
[1; 2; 3]        % column vector  (3×1)
[1 2; 3 4]       % 2×2 matrix
[1, 2, 3]        % commas work too
```

A bare newline inside `[...]` is a row separator, identical to `;`:

```
A = [1 2 3
     4 5 6]      % same as [1 2 3; 4 5 6]

v = [10
     20
     30]         % column vector (3×1)
```

Trailing `%` comments on a row are stripped before the newline is interpreted:

```
B = [100 200  % first row
     300 400] % second row
```

Line continuation (`...`) joins the next line into the **same** row — no row break occurs:

```
D = [1 2 ...
     3 4]         % same as [1 2 3 4]  (1×4 row vector)
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

### Matrix multiplication

`*` between two matrices performs standard matrix multiplication (inner
dimensions must agree):

```
A = [1 2; 3 4];
B = [1 0; 0 1];
A * B             % → same as A (multiply by identity)

v = [1; 2; 3];
v' * v            % dot product → 14 (1×3 times 3×1 = 1×1)
v * v'            % outer product → 3×3 matrix
```

### Transpose

Postfix `'` transposes a matrix. It binds tighter than any binary operator:

```
A'                % transpose of A
[1 2 3]'          % row vector → column vector (3×1)
R' * R            % for orthogonal R: gives identity
```

### Element-wise operators

`.*`, `./`, `.^` apply the operation to each pair of corresponding elements
(shapes must match):

```
A .* B            % element-wise product  (Hadamard product)
A ./ B            % element-wise division
A .^ 2            % square every element
v .^ 2            % same as v .* v
```

Note: `*` is matrix multiplication; `.*` is element-wise.

## Range operator

Generate row vectors with the `:` operator. Range has lower precedence than
arithmetic, so `1+1:5` evaluates as `2:5`.

```
1:5              % [1 2 3 4 5]
1:2:9            % [1 3 5 7 9]   (start:step:stop)
0:0.5:2          % [0 0.5 1 1.5 2]
5:-1:1           % [5 4 3 2 1]
5:1              % []   (empty — step in wrong direction)
```

Ranges work inside matrix literals — they are concatenated horizontally:

```
[1:4]            % [1 2 3 4]
[0, 1:3, 10]     % [0 1 2 3 10]
[1:2:7]          % [1 3 5 7]
[1:3; 4:6]       % 2×3 matrix: [1 2 3; 4 5 6]
```

### linspace

`linspace(a, b, n)` generates `n` evenly spaced values from `a` to `b`
(both endpoints included):

```
linspace(0, 1, 5)      % [0  0.25  0.5  0.75  1]
linspace(1, 5, 5)      % [1  2  3  4  5]
linspace(0, 1, 1)      % [1]   (single element returns b)
linspace(0, 1, 0)      % []   (empty)
```

## Built-in functions

| Function        | Description                              |
|-----------------|------------------------------------------|
| `zeros(m, n)`   | m×n matrix of zeros                      |
| `zeros(n)`      | n×n matrix of zeros                      |
| `ones(m, n)`    | m×n matrix of ones                       |
| `ones(n)`       | n×n matrix of ones                       |
| `eye(n)`        | n×n identity matrix                      |
| `size(A)`       | `[rows cols]` as a 1×2 row vector        |
| `size(A, dim)`  | Rows (dim=1) or columns (dim=2) as scalar|
| `length(A)`     | `max(rows, cols)`                        |
| `numel(A)`      | Total element count                      |
| `trace(A)`      | Sum of diagonal elements                 |
| `det(A)`        | Determinant (square matrices only)       |
| `inv(A)`        | Inverse (square, non-singular)           |

```
eye(3)            % 3×3 identity
det([1 2; 3 4])   % → -2
inv([1 2; 3 4])   % → 2×2 inverse matrix
size([1 2 3])     % → [1  3]
numel(zeros(3,4)) % → 12
```

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

## Indexing

All indices are **1-based** (Octave/MATLAB convention).
If a name exists as a variable in the workspace, `name(...)` is always
treated as indexing — variables shadow built-in function names.

### Vector indexing

```
v = [10 20 30 40 50];

v(3)         % → 30          scalar element
v(2:4)       % → [20 30 40]  sub-vector via range
v(:)         % → [10;20;30;40;50]  all elements, column vector
```

### Matrix indexing

```
A = [1 2 3; 4 5 6; 7 8 9];

A(2, 3)      % → 6           scalar at row 2, col 3
A(1, :)      % → [1 2 3]     entire row 1   (1×3)
A(:, 2)      % → [2;5;8]     entire column 2  (3×1)
A(1:2, 2:3)  % → [2 3; 5 6]  submatrix
```

### Index expressions

Index arguments can be arbitrary expressions:

```
n = size(A, 2);   % number of columns
A(1, n)           % last element of row 1
A(1:2, 1+1)       % rows 1-2, column 2
```

### `end` keyword

Inside any index expression, `end` resolves to the size of the dimension
being indexed. Arithmetic on `end` is supported.

```
v = [10 20 30 40 50];
v(end)           % → 50          last element
v(end-1)         % → 40          second to last
v(end-2:end)     % → [30 40 50]  last three

A = [1 2 3; 4 5 6; 7 8 9];
A(end, :)        % → [7 8 9]     last row
A(:, end)        % → [3;6;9]     last column
A(1:end-1, 2:end) % → [2 3; 5 6] all but last row, columns 2 onward
```

## Indexed Assignment

All index forms that work for reading also work for writing. The right-hand
side can be a single scalar (broadcast to all selected positions) or a
matrix/vector matching the selected size.

### Scalar and slice assignment

```
v = zeros(1, 6);
v(3) = 42;            % set one element
v(1:2) = [10, 20];    % set a slice from a vector
v(4:6) = 99;          % broadcast scalar to three positions
v(:) = 0;             % reset all elements at once
```

### 2-D matrix assignment

```
A = zeros(4);
A(2, 3) = 7;               % single element
A(:, 1) = [1; 2; 3; 4];   % entire column
A(1, :) = [10, 20, 30, 40]; % entire row
A(2:3, 2:3) = eye(2);      % submatrix
```

### Growing vectors

Assigning beyond the current length extends the vector and fills gaps with
zeros. `end+1` is the canonical Octave idiom for appending:

```
squares = [];
for k = 1:8
  squares(end+1) = k^2;
end
% squares = [1 4 9 16 25 36 49 64]

v = [1, 2, 3];
v(7) = 99;   % → [1 2 3 0 0 0 99]  (zeros fill the gap)
```

Assigning to a non-existent variable creates a new 1×N row vector:

```
fib(1) = 1;   % creates a 1×1 vector
fib(2) = 1;   % extends to 1×2
for k = 3:10
  fib(end+1) = fib(end) + fib(end-1);
end
```

### Logical (boolean mask) indexing

A 0/1 vector whose length equals the dimension selects positions where the
mask is 1. Masks can be produced by any comparison expression.

```
temps = [18, 22, 35, 12, 29, 41, 8, 33];

% Read: extract elements where mask is true
hot = temps(temps >= 30);   % → [35 41 33]

% Write: modify elements where mask is true
temps(temps >= 30) = 30;    % cap all hot days at 30

% Using a separate mask variable
mask = signal < 0;
signal(mask) = 0;           % half-wave rectifier
```

2-D matrices support logical masks as well — elements are selected in
column-major order (same as Octave/MATLAB):

```
M = [1 2 3; 4 5 6; 7 8 9];
M(M > 5)        % → [7 8 6 9]   (column-major order)
M(M > 5) = 0;   % zero out those elements
```

## Row separators inside matrix literals

Both `;` and bare newlines act as row separators inside `[...]`; they are never
statement separators there:

```
A = [1 2; 3 4];   % ; after ] suppresses output; ; inside is part of the matrix
B = [1 2
     3 4];        % newline inside [...] is a row separator; ; after ] suppresses output
```
