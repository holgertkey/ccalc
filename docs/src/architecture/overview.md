# Architecture Overview

## Workspace layout

```
ccalc/
├── Cargo.toml                  ← [workspace] — single version source
├── crates/
│   ├── ccalc/                  ← binary crate (CLI)
│   │   └── src/
│   │       ├── main.rs         ← entry point, mode detection
│   │       ├── repl.rs         ← REPL loop, pipe mode, evaluate()
│   │       └── help.rs         ← help text
│   └── ccalc-engine/           ← library crate (computation)
│       ├── src/
│       │   ├── lib.rs          ← public API
│       │   ├── env.rs          ← Env/Value types, workspace save/load
│       │   ├── eval.rs         ← AST + evaluator + formatters + Base enum
│       │   ├── parser.rs       ← tokenizer + recursive-descent parser, Stmt enum
│       │   ├── exec.rs         ← block executor, exec_stmts(), BODY_CHUNK_CACHE
│       │   ├── io.rs           ← IoContext — file descriptor table for fopen/fgetl/etc.
│       │   └── vm/             ← bytecode compiler + stack VM (Phase 34b)
│       │       ├── mod.rs      ← Opcode, Instr (8 B), Chunk, IterState, CompileError
│       │       ├── compile.rs  ← compile(&[StmtEntry]) → Chunk; is_compilable()
│       │       └── exec.rs     ← vm_exec(chunk, env, …)
│       └── benches/
│           └── engine.rs       ← Criterion benchmark suite
└── docs/                       ← this mdBook
```

## Data flow

```
User input (String)
    │
    ▼
parser::parse_stmts(input) → Vec<StmtEntry>   (AST — unchanged)
    │
    ▼
vm::compile::compile(&stmts)
    │  Ok(Chunk) ──────────────────────────────┐
    │  Err(Unsupported)                         │
    ▼                                           ▼
exec::exec_stmts (tree-walker)           vm::exec::vm_exec(chunk, env, …)
    │                                           │
    └─────────────────────┬─────────────────────┘
                          ▼
                    Value → format → stdout
```

`exec_stmts` attempts to compile the statement block on every call.
If compilation succeeds the VM executes it without per-statement recursion;
otherwise the tree-walker runs unchanged.  The public API is unaffected.

## Module responsibilities

| Module | Responsibility |
|---|---|
| `main.rs` | Parse CLI args, detect stdin mode (REPL / pipe / file / arg), dispatch |
| `repl.rs` | REPL event loop, pipe line-reader, shared `evaluate()`, display logic |
| `help.rs` | Static help text |
| `env.rs` | `Value` enum, `Env` type (`HashMap<String, Value>`), workspace save/load |
| `eval.rs` | `Expr` AST, `Op`, `Base`; evaluator, formatters, built-ins, `FnCallHook` |
| `parser.rs` | Tokenizer, recursive-descent parser, `parse_stmts()`, `Stmt` enum |
| `exec.rs` | `exec_stmts()`, `exec_script()`, `Signal`, user function dispatch, `BODY_CHUNK_CACHE` |
| `io.rs` | `IoContext` — file descriptor table for `fopen`/`fgetl`/`fprintf`/etc. |
| `vm/mod.rs` | `Opcode`, `Instr` (8-byte fixed-width), `Chunk`, `IterState`, `CompileError` |
| `vm/compile.rs` | `compile(&[StmtEntry]) → Result<Chunk, CompileError>`; `is_compilable()` |
| `vm/exec.rs` | `vm_exec(chunk, env, io, …)` — bytecode dispatch loop, arithmetic fast paths |
| `benches/engine.rs` | Criterion benchmarks: scalar ops, fib, loop throughput, matmul, fn calls |

## Dependency graph

```
ccalc (binary)
  ├── ccalc-engine (local)
  │     ├── dirs
  │     ├── ndarray
  │     └── indexmap
  ├── rustyline
  ├── toml
  └── serde

ccalc-engine (dev / benches only)
  └── criterion
```

## Design principles

- **One binary, no runtime.** The release binary is self-contained.
  Every new dependency requires explicit justification.
- **The library is pure.** `ccalc-engine` has no I/O, no terminal codes,
  no `rustyline`. The binary owns all user-facing interaction.
- **Modern MATLAB standard (R2016b+).** Where MATLAB and Octave differ,
  ccalc follows modern MATLAB. Example: `'text'` is a char array (numeric),
  `"text"` is a string object.
- **Version is defined once** in `[workspace.package]`; both crates inherit it.
- **No runtime allocations on the hot path** beyond the input string itself.
