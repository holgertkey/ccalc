# Matrix Utilities & Set Operations

Phase 23 adds triangular-matrix extraction, tiling, Kronecker products, vector
products, set-theoretic operations on vectors, and index-conversion utilities.

## Triangular extraction

```matlab
A = [1 2 3; 4 5 6; 7 8 9];

triu(A)        % [1 2 3; 0 5 6; 0 0 9]   upper triangular
triu(A, 1)     % [0 2 3; 0 0 6; 0 0 0]   above main diagonal
tril(A)        % [1 0 0; 4 5 0; 7 8 9]   lower triangular
tril(A, -1)    % [0 0 0; 4 0 0; 7 8 0]   below main diagonal
```

The optional offset `k`:

| `k` | keeps elements where … |
|-----|------------------------|
| `0` (default) | `col − row ≥ 0` (`triu`) / `col − row ≤ 0` (`tril`) |
| `k > 0` | strictly above the main diagonal |
| `k < 0` | extends into the sub-diagonals |

## Tiling and Kronecker product

```matlab
repmat([1 2; 3 4], 2, 3)            % 4×6 block matrix
kron([1 0; 0 1], [1 2; 3 4])        % 4×4 block-diagonal (identity scaling)
```

`repmat(A, m, n)` tiles matrix `A` in an `m × n` grid of blocks.

`kron(A, B)` replaces each scalar element `a[i,j]` of `A` with the block
`a[i,j] * B`, producing a `(rows_A × rows_B)` by `(cols_A × cols_B)` result.

## Vector products

```matlab
cross([1 0 0], [0 1 0])   % [0 0 1]
cross([1 2 3], [4 5 6])   % [-3 6 -3]

dot([1 2 3], [4 5 6])     % 32
```

`cross(a, b)` requires both vectors to have exactly 3 elements.  The result
orientation (row or column) matches argument `a`.

`dot(a, b)` computes the inner product `sum(a .* b)` and returns a scalar.

## Set operations

All set functions return sorted, unique results.  NaN is never considered a
member (IEEE semantics: `NaN ≠ NaN`).

```matlab
intersect([1 3 5 7], [3 5 9])    % [3 5]
union([1 3 5], [3 5 7])          % [1 3 5 7]
setdiff([1 2 3 4 5], [2 4])      % [1 3 5]

ismember(3, [1 2 3 4])           % 1
ismember([1 6 3], [1 2 3 4])     % [1 0 1]  (element-wise)
ismember(nan, [nan])             % 0  (NaN is never a member)
```

## Index conversion

`sub2ind` and `ind2sub` convert between row/column subscripts and 1-based
column-major linear indices (MATLAB convention).

```matlab
sub2ind([3 4], 2, 3)            % 8    (scalar)
sub2ind([3 4], [1 2], [1 3])    % [1 8]  (vectorised)

[r, c] = ind2sub([3 4], 8)      % r=2, c=3
[r, c] = ind2sub([3 4], [1 7])  % r=[1 1], c=[1 3]
```

## Element repetition

```matlab
repelem([1 2 3], 3)          % [1 1 1 2 2 2 3 3 3]
repelem([1 2 3], [2 1 3])    % [1 1 2 3 3 3]  (per-element counts)
repelem([1 2; 3 4], 2, 3)    % 4×6  (each element repeated 2 rows × 3 cols)
```

`repelem(v, n)` — repeat each element `n` times (scalar `n`).  
`repelem(v, nv)` — repeat `v(i)` by `nv(i)` times (vector `nv`).  
`repelem(A, m, n)` — 2-D form: repeat each element `m` rows and `n` columns.

## See also

- [`help matrices`](./matrices.md) — matrix literals and arithmetic
- [`help vectors`](./vectors.md) — sort, find, unique, reshape
- [`help linalg`](./linear-algebra.md) — QR, LU, SVD, eigenvectors
