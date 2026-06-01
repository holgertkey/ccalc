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

// Value enum — result of evaluation
pub enum env::Value {
    Scalar(f64), Matrix(Array2<f64>), Complex(f64, f64),
    ComplexMatrix(Array2<Complex<f64>>), Str(String), StringObj(String),
    Lambda(LambdaFn), Function { .. }, Cell(Vec<Value>),
    Struct(IndexMap<String, Value>), StructArray(Vec<IndexMap<..>>),
    Map(IndexMap<String, Value>), Void,
    DateTime(f64), Duration(f64), DateTimeArray(Vec<f64>), DurationArray(Vec<f64>),
    Tuple(Vec<Value>),
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

## Why a separate crate?

- **Testable in isolation** — 1 000+ unit tests, no CLI coupling.
- **Embeddable** — WASM or other frontends can link `ccalc-engine` directly.
- **Clean boundary** — the binary owns all user-facing interaction;
  the engine has no `rustyline`, no terminal codes, no `println!` in hot paths.
