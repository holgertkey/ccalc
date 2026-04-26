# Phase 20a — JSON

Introduced in **v0.24.0**.

Phase 20a adds `jsondecode` and `jsonencode` built-ins, backed by `serde_json`,
behind an optional `json` feature flag. The default binary is unaffected.

---

## Feature flag

```bash
# Build with JSON support:
cargo build --release --features json

# Pass through from the top-level workspace:
cargo build --release --features json
```

The `json` feature is declared in both `ccalc-engine/Cargo.toml` and
`ccalc/Cargo.toml` (as a pass-through). The engine crate adds
`serde_json = { version = "1", optional = true }` as an optional dependency.

When the feature is **disabled**, calling either built-in returns:

```
jsondecode: not available — rebuild with --features json
```

Both names still appear in tab completion regardless of the feature flag.

---

## Built-ins

### `jsondecode(str)`

Parses the JSON string `str` and returns a ccalc `Value`.

**Mapping:**

| JSON | ccalc |
|------|-------|
| object | `Struct` (fields in insertion order via `IndexMap`) |
| all-numbers array (+ nulls) | `Matrix` 1×N row vector (`null` → `NaN`) |
| mixed array | `Cell` |
| string | `Str` |
| number | `Scalar` |
| `true` / `false` | `Scalar(1.0 / 0.0)` |
| `null` | `Scalar(NaN)` |

Errors on invalid JSON (`"jsondecode: invalid JSON: …"`).
Errors if the argument is not a string (`"jsondecode: argument must be a string"`).

### `jsonencode(val)`

Encodes a ccalc `Value` to a compact JSON string (`Value::Str`).

**Mapping:**

| ccalc | JSON |
|-------|------|
| `Struct` | object |
| `Matrix` 1×N | flat array |
| `Matrix` M×N | array of row arrays |
| `Cell` | array |
| `StructArray` | array of objects |
| `Scalar(NaN)` | `null` |
| `Scalar(finite)` | number |
| `Str` / `StringObj` | string |

Errors for `Complex`, `Lambda`, `Function`, `Void`, `Tuple`, and `Scalar(±Inf)`.

---

## Implementation

- **`crates/ccalc-engine/src/json.rs`** — new module (compiled only under `#[cfg(feature = "json")]`). Contains:
  - `pub(crate) fn json_to_value(v: &serde_json::Value) -> Value`
  - `pub(crate) fn value_to_json(v: &Value) -> Result<serde_json::Value, String>`
  - `fn decode_array(arr: &[serde_json::Value]) -> Value` — separates all-numeric from mixed arrays
  - `fn encode_f64(x: f64) -> Result<serde_json::Value, String>` — handles `NaN → null`, `Inf → error`

- **`crates/ccalc-engine/src/eval.rs`** — changes:
  - `jsondecode` and `jsonencode` added to `builtin_names()` (unconditional — always in tab completion)
  - `("jsondecode", 1)` and `("jsonencode", 1)` match arms in `call_builtin` dispatch to `jsondecode_impl` / `jsonencode_impl`
  - `jsondecode_impl` / `jsonencode_impl`: dual `#[cfg(feature = "json")]` / `#[cfg(not(feature = "json"))]` implementations

- **`crates/ccalc-engine/src/lib.rs`** — `#[cfg(feature = "json")] pub(crate) mod json;`

- **`crates/ccalc/Cargo.toml`** — `json = ["ccalc-engine/json"]` feature pass-through

---

## Tests

22 tests in `eval_tests.rs` under `mod json_tests` (gated with `#[cfg(feature = "json")]`):

```bash
cargo test --features json
```

Test coverage: scalar decode, null→NaN, bool→0/1, string, numeric array, empty array,
mixed→Cell, object→Struct, nested struct, invalid JSON error, non-string arg error,
scalar encode, NaN→null, Inf error, string, row vector, 2-D matrix, struct, cell,
and a full roundtrip test.
