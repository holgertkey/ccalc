# Phase 18 — Advanced Linear Algebra

**Version:** v0.22.0

Pure-Rust implementations — no BLAS/LAPACK dependency required.
All new functions extend the existing `call_builtin` dispatch in `eval.rs`.

---

## 18a — QR decomposition

**Algorithm:** Householder reflectors applied from the left.
For each column k, a reflector `H_k` zeroes the sub-diagonal entries.
`Q = H_1 * H_2 * ... * H_k` is accumulated as a full m×m orthogonal matrix.

```rust
fn qr_decompose(a: &Array2<f64>) -> Result<(Array2<f64>, Array2<f64>), String>
```

**Interface:**

```
[Q, R] = qr(A)   % A = Q * R; Q: m×m orthogonal, R: m×n upper triangular
R = qr(A)        % single-output: R only
```

`get_nargout()` selects between single-value and tuple return.

---

## 18b — LU decomposition

**Algorithm:** Gaussian elimination with partial pivoting.
At each step, the row with the largest absolute pivot is swapped into position.

```rust
fn lu_decompose(a: &Array2<f64>) -> Result<(Array2<f64>, Array2<f64>, Array2<f64>), String>
```

Returns `(L, U, P)` — unit lower triangular, upper triangular, permutation matrix
(as a dense `Array2<f64>`, column-major encoding).

**Interface:**

```
[L, U, P] = lu(A)   % PA = LU
U = lu(A)           % single-output: U only
```

---

## 18c — Cholesky decomposition

**Algorithm:** Standard row-by-row Cholesky. Returns an error if any diagonal
entry would be ≤ 0 (i.e., the matrix is not positive definite).

```rust
fn chol_decompose(a: &Array2<f64>) -> Result<Array2<f64>, String>
```

Returns upper triangular `R` such that `R' * R = A`.

**Interface:**

```
R = chol(A)   % errors if A is not symmetric positive definite
```

---

## 18d — SVD

**Algorithm:** One-sided Jacobi SVD (Golub–Van Loan convention).
Iterates Givens rotations on pairs of columns `(p, q)` until orthogonality
is achieved: `γ² ≤ ε² * α * β` where `α = ‖b_p‖²`, `β = ‖b_q‖²`, `γ = b_p · b_q`.
Maximum 200 sweeps; convergence guaranteed for well-conditioned matrices.

For `m < n` the input is transposed and U/V swapped on return.

```rust
fn svd_compute(a: &Array2<f64>) -> Result<(Array2<f64>, Vec<f64>, Array2<f64>), String>
```

Returns `(U_economy, s_vec, V_economy)` — economy form.

**Full SVD** extends `U` to m×m via `complete_orthonormal_basis` (Gram-Schmidt
against the existing columns, using standard basis vectors as candidates).
`S` is built as a full m×n diagonal matrix.

**Interface:**

```
s = svd(A)                % singular values as a column vector
[U, S, V] = svd(A)        % full SVD: U (m×m), S (m×n), V (n×n); A = U*S*V'
[U, S, V] = svd(A,'econ') % economy: U (m×k), S (k×k), V (n×k); k = min(m,n)
```

---

## 18e — Eigendecomposition

**Algorithm:** QR iteration with Wilkinson shift.
Shifts converge cubically for symmetric matrices.

Wilkinson shift for the trailing 2×2 submatrix:

```
δ = (a - d) / 2
μ = d - b² / (δ + sign(δ) · √(δ² + b²))     (δ ≠ 0)
μ = d - |b|                                   (δ = 0)
```

Each iteration: subtract μI, deflate (zero sub-diagonal entries below threshold),
then QR-step and re-add μI. Eigenvectors are accumulated via the Q factors.

```rust
fn eig_compute(a: &Array2<f64>) -> Result<(Vec<f64>, Array2<f64>), String>
```

Returns `(eigenvalues, eigenvectors)`.

**Interface:**

```
d = eig(A)        % eigenvalues as a column vector
[V, D] = eig(A)   % V: eigenvectors (columns), D: diagonal eigenvalue matrix
```

---

## 18f — Matrix properties

All five functions are thin wrappers over `svd_compute`.

| Function   | Description                                                       |
|------------|-------------------------------------------------------------------|
| `rank(A)`  | Count of singular values > `ε * s_max * max(m, n)` (ε = 2.2e-16) |
| `null(A)`  | Right singular vectors corresponding to near-zero singular values |
| `orth(A)`  | Left singular vectors corresponding to non-negligible svals       |
| `cond(A)`  | `s_max / s_min`; `Inf` if any singular value is zero              |
| `pinv(A)`  | `V * diag(1/sᵢ for sᵢ > ε) * U'`                                 |

---

## 18g — Updated `norm`

Previously `norm(A)` for a non-vector matrix returned an error.

| Call               | Result                                     |
|--------------------|--------------------------------------------|
| `norm(A)`          | Largest singular value (spectral 2-norm)   |
| `norm(A, 'fro')`   | `sqrt(sum of squared elements)`            |
| `norm(A, 1)`       | Max column-sum                             |
| `norm(A, inf)`     | Max row-sum                                |
| `norm(v)` / `norm(v,p)` | Vector Lp norm — unchanged           |

---

## `nargout` thread-local

Multi-output built-ins (`qr`, `lu`, `svd`, `eig`) return either a single
value or a `Value::Tuple` depending on `get_nargout()`.

`set_nargout(n)` (public, in `eval.rs`) is called at two sites:

- **`exec_stmts`** (`exec.rs`): `Stmt::Assign` → 1; `Stmt::MultiAssign` → targets.len()
- **`evaluate()`** (`repl.rs`): `Stmt::Assign` → 1

This mirrors the `NARGOUT` thread-local pattern used by `FN_CALL_HOOK`,
`AUTOLOAD_HOOK`, and `RUN_DEPTH`.

---

## Tests

25 new tests in `crates/ccalc-engine/src/eval_tests.rs`:

- QR: orthogonality of Q, reconstruction A = Q*R
- LU: PA = LU verification for 3×3 and ill-conditioned matrices
- Cholesky: R'*R = A for SPD; error for non-SPD
- SVD: singular values, U/V orthogonality, reconstruction A = U*S*V'
- Eigendecomposition: A*V = V*D residual < 1e-10
- rank/null/orth/cond/pinv: correctness and fundamental properties
- Matrix norms: 2-norm, Frobenius, column-sum, row-sum

## Example

See `examples/linear_algebra.calc` for a full demo covering all Phase 18
functions with mathematical verification of each result.
