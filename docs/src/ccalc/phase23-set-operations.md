# Phase 23 — Matrix Utilities & Set Operations

**Version:** 0.28.0  
**Prerequisites:** Phase 7.5 (basic matrix utilities), Phase 15 (indexed assignment), Phase 18 (linear algebra context)

Pure built-in additions — no new tokens, AST nodes, or `Value` variants.

## 23a — Triangular extraction and tiling

| Function | Description |
|---|---|
| `triu(A)` / `triu(A, k)` | Upper triangular; zero elements where `col − row < k` |
| `tril(A)` / `tril(A, k)` | Lower triangular; zero elements where `col − row > k` |
| `repmat(A, m, n)` | Tile `A` in an `m × n` block grid |
| `kron(A, B)` | Kronecker product |

## 23b — Vector products

| Function | Description |
|---|---|
| `cross(a, b)` | Cross product of two length-3 vectors; result orientation matches `a` |
| `dot(a, b)` | Inner product `sum(a .* b)` → scalar |

## 23c — Set operations

Results are always sorted ascending and contain no duplicates.  NaN is never a
member (IEEE semantics).

| Function | Description |
|---|---|
| `intersect(a, b)` | Elements present in both vectors |
| `union(a, b)` | All unique elements from both vectors |
| `setdiff(a, b)` | Elements of `a` not in `b` |
| `ismember(x, v)` | `1` if `x` ∈ `v`; element-wise for vector `x` |

## 23d — Index utilities and element repetition

| Function | Description |
|---|---|
| `sub2ind(sz, r, c)` | Subscripts → linear index (1-based, column-major) |
| `ind2sub(sz, idx)` | Linear index → `[r; c]` tuple |
| `repelem(v, n)` | Repeat each element of `v` exactly `n` times |
| `repelem(v, nv)` | Repeat `v(i)` by `nv(i)` times |
| `repelem(A, m, n)` | 2-D: repeat each element `m` rows × `n` cols |
