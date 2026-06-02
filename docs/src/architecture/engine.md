# Engine Crate (`ccalc-engine`)

The `ccalc-engine` crate is a pure computation library with no terminal
I/O dependencies.  It contains the full language pipeline.

## Execution pipeline

```
parse_stmts(src) → Vec<StmtEntry>   (AST)
        │
        ▼
vm::compile::compile(&stmts)
        │   Ok(Chunk)              Err(Unsupported)
        ▼                                  ▼
vm::exec::vm_exec(chunk, env, …)   exec::exec_stmts (tree-walker)
```

`exec_stmts` is the public execution entry point.  It tries to compile the
statement block to bytecode first; if any construct is not yet supported
(`CompileError::Unsupported`), it falls back to the recursive tree-walker
transparently.

## Key public types

```rust
// Statement AST — produced by the parser
pub enum parser::Stmt { Assign(..), Expr(..), For { .. }, While { .. }, … }
pub type  parser::StmtEntry = (Stmt, /*silent*/ bool, /*line*/ usize);

// Value enum — result of evaluation  (sizeof = 32 bytes, Phase 35b)
pub enum env::Value {
    // ── unboxed (small) ────────────────────────────────────────────────
    Void,
    Scalar(f64),
    Complex(f64, f64),
    DateTime(f64),    Duration(f64),
    Str(String),      StringObj(String),
    Tuple(Vec<Value>), DateTimeArray(Vec<f64>), DurationArray(Vec<f64>),
    // ── boxed (large, one heap pointer each) ──────────────────────────
    Matrix(Box<Array2<f64>>),
    ComplexMatrix(Box<Array2<Complex<f64>>>),
    Function(Box<FunctionData>),        // outputs, params, body_source, locals, doc
    Lambda(Box<LambdaFn>),
    Cell(Box<Vec<Value>>),
    Struct(Box<IndexMap<String, Value>>),
    StructArray(Box<Vec<IndexMap<String, Value>>>),
    Map(Box<IndexMap<String, Value>>),
}

// Associated struct for named user functions (behind Box in Value::Function)
pub struct env::FunctionData {
    pub outputs:     Vec<String>,
    pub params:      Vec<String>,
    pub body_source: String,
    pub locals:      IndexMap<String, Value>,
    pub doc:         Option<String>,
}

// Variable environment
pub type env::Env = IndexMap<String, Value>;

// Execute a parsed block (tries VM, falls back to tree-walker)
pub fn exec::exec_stmts(stmts, env, io, fmt, base, compact)
    -> Result<Option<Signal>, String>;

// Execute a top-level script (hoists function defs, then exec_stmts)
pub fn exec::exec_script(stmts, env, io, fmt, base, compact)
    -> Result<Option<Signal>, String>;
```

## Bytecode VM (`vm/`)

Added in Phase 34b.  Three modules:

| Module | Role |
|--------|------|
| `vm/mod.rs` | Shared types: `Opcode` (u8), `Instr` (8 bytes, compile-time size assert), `Chunk`, `IterState`, `CompileError` |
| `vm/compile.rs` | `compile(&[StmtEntry]) → Result<Chunk, CompileError>` — single-pass lowering; `is_compilable(&[StmtEntry]) → bool` — zero-allocation pre-check |
| `vm/exec.rs` | `vm_exec(chunk, env, io, fmt, base, compact) → Result<Option<Signal>, String>` — main dispatch loop |

`Instr` is always 8 bytes: 1-byte opcode + 7-byte little-endian payload.
This fits thousands of instructions in L1-D cache.

Supported compiled statements: `Assign`, `Expr`, `For`, `While`, `If`/elseif/else,
`Break`, `Continue`, `Return`, `FunctionDef` (→ `DefineFunc`), `IndexSet`
(→ `IndexSetOp`).

Arithmetic fast paths: Scalar×Scalar (direct `f64`), Complex power via
`num_complex::powi`/`powf`/`powc`, Matrix broadcast via `ndarray`.

## Phase 35 — Interpreter Performance 2

Three sub-phases reduced loop overhead from ~4.7 ms/10k-iter to ~0.56 ms:

### 35a — Slot-indexed locals

Variables that are only assigned in the current chunk and never referenced
inside an `EvalExpr` expression receive consecutive **slot indices** instead of
`HashMap` keys.  New opcodes `LoadSlot`/`StoreSlot`/`IterNextSlot` access a
`Vec<Value>` by integer index — O(1) with zero hashing.  The compiler performs
two passes: collect assignment-LHS/loop-var candidates, filter out any name
that appears free inside an `EvalExpr` sub-expression, assign slots to the rest.
Entry and exit of `vm_exec` sync slots to/from `env` in O(slots) passes.

### 35c — Native `CallBuiltin` opcode

A `COMPILABLE_BUILTINS` whitelist (57 pure-math functions: `abs`, `sqrt`,
`sin`/`cos`, `real`/`imag`, `sum`, `size`, `zeros`, …) marks calls as pure.
`is_pure()` returns `true` for whitelisted calls, so their arguments are no
longer EvalExpr-referenced.  The `CallBuiltin(name_idx, argc)` opcode pops
arguments directly from the VM stack and calls `call_builtin` — no env lookup,
no AST traversal.

**Side-effect:** once `abs(z)` becomes `CallBuiltin`, `z` is no longer
EvalExpr-referenced → 35a assigns it a slot → Julia-set inner loop is
fully slot-indexed.

### 35b — Value boxing

`sizeof(Value)` reduced from **168 → 32 bytes** by placing eight large variants
behind `Box<T>` (see the `Value` enum listing above).  Benefits:

| Impact | Detail |
|--------|--------|
| Slot `Vec<Value>` | 5–7× smaller; fits in a single cache line for typical functions |
| VM operand stack | Same reduction; `push`/`pop` memcopy 32 B not 168 B |
| `for k = 1:256` iterator | 256 × 32 B = 8 KB (was 43 KB) |

A compile-time assertion `const _VALUE_SIZE: () = assert!(size_of::<Value>() <= 32)`
prevents future size regressions.

### Benchmark summary (release, Windows 11)

| Benchmark | v0.45 (Phase 34b) | v0.46 (Phase 35) | Improvement |
|-----------|-------------------|------------------|-------------|
| `loop_10k` | 4.68 ms | **0.56 ms** | 8.4× |
| `fn_calls_1000` | 3.10 ms | 2.92 ms | 1.06× |
| `scalar_ops_sum_1M` | 8.05 ms | 9.40 ms | within budget |

`fn_calls_1000` is bounded by `call_user_function` (fresh `Env` allocation per
call), not by the inner loop — a separate optimisation target.

## Why a separate crate?

- **Testable in isolation** — 1 000+ unit tests, no CLI coupling.
- **Embeddable** — WASM or other frontends can link `ccalc-engine` directly.
- **Clean boundary** — the binary owns all user-facing interaction;
  the engine has no `rustyline`, no terminal codes, no `println!` in hot paths.
