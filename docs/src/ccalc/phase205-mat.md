# Phase 20.5 — MAT File Read

Introduced in **v0.25.0**.

Phase 20.5 adds `load('file.mat')` support, backed by `matrw = "=0.1.4"`,
behind an optional `mat` feature flag. The default binary is unaffected.

---

## Feature flag

```bash
# Build with MAT support:
cargo build --release --features mat

# Run a script that uses load('*.mat'):
cargo run --release --features mat -- examples/mat/mat.calc
```

The `mat` feature is declared in both `ccalc-engine/Cargo.toml` and
`ccalc/Cargo.toml` (as a pass-through). The engine crate adds
`matrw = { version = "=0.1.4", optional = true }` as an optional dependency.

When the feature is **disabled**, calling `load('*.mat')` returns:

```
load: .mat support not available — rebuild with --features mat
```

The `load` built-in always appears in tab completion regardless of the feature flag.

---

## Built-in: `load`

### Assignment form

```matlab
data = load('results.mat')
```

Returns a `Struct` whose fields are the variable names stored in the file.

```matlab
data = load('examples/mat/fixtures/sample.mat');
data.score        % → 92.5    (Scalar)
data.label        % → 'experiment-1'  (Str)
data.readings     % → [23.1  21.8  24.3  ...]  (1×6 Matrix)
data.sensor.gain  % → 0.5    (nested Struct field)
```

### Bare form

```matlab
load('results.mat')
```

Merges all variables from the file directly into the current workspace — each
variable name becomes a variable in the calling scope.

```matlab
load('examples/mat/fixtures/sample.mat')
score          % → 92.5
readings       % → [23.1  21.8  ...]
sensor.gain    % → 0.5
```

### `save` with `.mat` extension

Writing `.mat` files is not yet implemented. `save('out.mat', ...)` returns a
clear error message instead of silently producing a corrupt file.

---

## Type mapping

| MAT type | ccalc `Value` |
|----------|---------------|
| `double` (1×1 scalar) | `Scalar` |
| `double` (M×N matrix) | `Matrix` (column-major → row-major conversion) |
| `char` array | `Str` |
| `struct` (scalar) | `Struct` |
| struct array (1 element) | `Struct` (unwrapped) |
| struct array (N elements) | `StructArray` |
| `cell` array | `Cell` |
| `[]` / null | `Scalar(NaN)` |

Complex and sparse matrices are not yet supported and produce an error.

---

## Implementation

- **`crates/ccalc-engine/src/mat.rs`** — new module (compiled only under
  `#[cfg(feature = "mat")]`). Contains:
  - `pub(crate) fn mat_load(path: &str) -> Result<Value, String>` — iterates
    over `matrw::load_matfile()` entries and builds a `Value::Struct`.
  - `fn mat_var_to_value(var: &MatVariable) -> Result<Value, String>` — recursive
    converter: `NumericArray` → `Scalar`/`Matrix`/`Str`, `Structure` → `Struct`,
    `StructureArray` → `Struct`/`StructArray`, `CellArray` → `Cell`, `Null` → `Scalar(NaN)`.
  - Column-major conversion: `Array2::from_shape_vec((cols, rows), data).t().to_owned()`.

- **`crates/ccalc-engine/src/eval.rs`** — changes:
  - `"load"` added to `builtin_names()` (unconditional).
  - `("load", 1)` match arm in `call_builtin` dispatches to `load_mat_file()`.
  - `pub fn load_mat_file(path)` with dual `#[cfg]`/`#[cfg(not)]` stubs, so
    `repl.rs` can call it without importing the `mat` module.

- **`crates/ccalc-engine/src/lib.rs`** — `#[cfg(feature = "mat")] pub(crate) mod mat;`

- **`crates/ccalc/Cargo.toml`** — `mat = ["ccalc-engine/mat"]` feature pass-through.

- **`crates/ccalc/src/repl.rs`** — `.mat` extension check in both REPL and pipe
  `SaveLoadCmd::Load` handlers: injects each field of the returned `Struct` into
  the workspace. `SaveLoadCmd::Save` with `.mat` path emits an error.

---

## Tests

5 roundtrip tests in `eval_tests.rs` under `mod mat_tests`
(gated with `#[cfg(feature = "mat")]`):

```bash
cargo test --features mat
```

Test coverage: scalar roundtrip, row-vector roundtrip, 2×3 matrix with
correct column-major conversion, char array → `Str`, nested struct fields.

A `create_example_fixture` test (marked `#[ignore]`) generates the fixture
used by the example script:

```bash
cargo test --features mat create_example_fixture -- --ignored
```

---

## Example

```bash
cargo run --release --features mat -- examples/mat/mat.calc
```

The script covers: assignment form, scalar access, row-vector statistics and
normalization, matrix display and algebra, char-array built-ins, struct field
access, bare form workspace merge, and a moving-average signal analysis.
