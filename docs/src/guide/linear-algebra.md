# Linear Algebra

ccalc supports a comprehensive set of matrix decompositions and properties,
implemented in pure Rust with no BLAS/LAPACK dependency.

All decompositions use `[a, b] = f(x)` multi-output assignment syntax.
Single-output forms are also available for convenience.

## QR decomposition

`qr(A)` factors a matrix as `A = Q * R`, where Q is orthogonal and R is
upper triangular.

```
[Q, R] = qr(A)    % Q: m×m orthogonal, R: m×n upper triangular
R = qr(A)         % single-output: R only
```

The full Q returned by ccalc is always m×m. For least-squares problems
with an overdetermined system, extract the "thin" (economy) factors:

```
A = [1 2; 3 4; 5 6];            % 3×2 overdetermined
[Q, R] = qr(A);
Q1 = Q(:, 1:2);                  % first n columns
R1 = R(1:2, :);                  % first n rows (2×2 square)

b = [1; 2; 3];
c = R1 \ (Q1' * b);              % least-squares solution
```

Verification:

```
norm(Q' * Q - eye(3), 'fro')     % ≈ 0  (Q orthogonal)
norm(Q * R - A, 'fro')           % ≈ 0  (exact factorisation)
```

## LU decomposition

`lu(A)` factors a square matrix with partial pivoting: **PA = LU**,
where P is a permutation matrix, L is unit lower triangular, and U is
upper triangular.

```
[L, U, P] = lu(A)   % PA = LU
U = lu(A)           % single-output: U only
```

```
B = [2, 1, -1; -3, -1, 2; -2, 1, 2];
[L, U, P] = lu(B);
norm(P * B - L * U, 'fro')       % ≈ 0

x = B \ [8; -11; -3];            % backslash uses LU internally
```

## Cholesky decomposition

`chol(A)` returns the upper triangular Cholesky factor R such that
**A = R' \* R**. The input must be symmetric positive definite (SPD).
An error is returned otherwise.

```
A = [4 2 2; 2 5 3; 2 3 6];
R = chol(A);
norm(R' * R - A, 'fro')          % ≈ 0
```

Cholesky is about twice as fast as LU for SPD systems and also serves as
a positive-definiteness test.

## Singular value decomposition (SVD)

`svd(A)` computes the decomposition **A = U \* S \* V'**.

```
s = svd(A)                       % singular values as a column vector (descending)
[U, S, V] = svd(A)               % full: U (m×m), S (m×n diagonal), V (n×n)
[U, S, V] = svd(A, 'econ')       % economy: U (m×k), S (k×k), V (n×k)
```

```
C = [1 2 3; 4 5 6; 7 8 9];       % rank-2 matrix
s = svd(C);
fprintf('rank = %d\n', rank(C))   % 2

[U, S, V] = svd(C);
norm(U * S * V' - C, 'fro')      % ≈ 0
norm(U' * U - eye(3), 'fro')     % ≈ 0  (U orthogonal)

% Rank-1 approximation (best in Frobenius sense)
C1 = S(1,1) * (U(:,1) * V(:,1)');
```

## Eigendecomposition

`eig(A)` returns eigenvalues and eigenvectors. Best results for symmetric
(Hermitian) matrices, which guarantee real eigenvalues.

```
d = eig(A)           % eigenvalues as a column vector
[V, D] = eig(A)      % V: eigenvectors (columns), D: diagonal eigenvalue matrix
```

The decomposition satisfies **A \* V = V \* D**, i.e.
`A * V(:,k) = D(k,k) * V(:,k)` for each eigenpair k.

```
S = [4 1 0; 1 3 1; 0 1 2];      % symmetric
[V, D] = eig(S);

% Check residual for each eigenpair
for k = 1:3
  r = norm(S * V(:,k) - D(k,k) * V(:,k));
  fprintf('residual %d: %.2e\n', k, r)
end
```

## Matrix properties

### Numerical rank

`rank(A)` counts the singular values above the threshold
`ε × σ_max × max(m, n)` (where ε = 2.2×10⁻¹⁶, the double precision machine epsilon).

```
rank([1 2 3; 4 5 6; 7 8 9])     % → 2  (third row is sum of first two)
rank(eye(4))                      % → 4
```

### Null space

`null(A)` returns an orthonormal basis for the null space of A —
the set of vectors x such that A\*x = 0.

```
N = null([1 2 3; 4 5 6; 7 8 9]);
norm(([1 2 3; 4 5 6; 7 8 9]) * N)   % ≈ 0
```

### Column-space basis

`orth(A)` returns an orthonormal basis for the column space of A
(the range or image of A).

```
Q = orth([1 2 3; 4 5 6; 7 8 9]);   % 3×2 (rank 2 matrix → 2 basis vectors)
norm(Q' * Q - eye(2), 'fro')        % ≈ 0  (Q has orthonormal columns)
```

### Condition number

`cond(A)` returns the 2-norm condition number σ_max / σ_min.
A large condition number means the matrix is nearly singular and linear
systems involving it may be sensitive to small perturbations.

```
cond(eye(3))                        % → 1.0  (perfectly conditioned)
cond([1 1; 1 1.0001])               % → ~40000  (nearly singular)
```

### Pseudoinverse

`pinv(A)` computes the Moore-Penrose pseudoinverse via SVD.
For full-rank square matrices it coincides with `inv(A)`.
For rank-deficient or non-square matrices it gives the minimum-norm
least-squares solution.

```
A = [1 2 3; 4 5 6; 7 8 9];         % rank 2
Ap = pinv(A);
norm(A * Ap * A - A, 'fro')         % ≈ 0  (fundamental property)
rank(Ap)                             % → 2  (same as rank(A))
```

## Matrix norms

| Call              | Description                                     |
|-------------------|-------------------------------------------------|
| `norm(v)`         | Vector Euclidean (L2) norm                      |
| `norm(v, p)`      | Vector Lp norm                                  |
| `norm(A)`         | Matrix spectral 2-norm (largest singular value) |
| `norm(A, 'fro')`  | Frobenius norm: √(Σ aᵢⱼ²)                      |
| `norm(A, 1)`      | Max column-sum norm                             |
| `norm(A, inf)`    | Max row-sum norm                                |

```
M = [1 2; 3 4; 5 6];
norm(M)           % 9.5255  (largest singular value)
norm(M, 'fro')    % 9.5394  (sqrt(1+4+9+16+25+36))
norm(M, 1)        % 12.0    (max column sum: max(1+3+5, 2+4+6))
norm(M, inf)      % 11.0    (max row sum: max(1+2, 3+4, 5+6))
```

## Tip: negative elements in matrix literals

A space before a minus sign inside `[...]` can be parsed as subtraction
rather than a negative element. Use commas to be unambiguous:

```
A = [2, 1, -1; -3, -1, 2]   % safe: comma disambiguates
A = [2 1 -1; ...]            % risky: '1 -1' parses as 1 - 1 = 0
```

## Example

```
ccalc examples/linear_algebra.calc
```

The example script covers all functions above with numerical verification
of each decomposition and matrix property.
