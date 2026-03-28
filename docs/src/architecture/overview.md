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
│       └── src/
│           ├── lib.rs          ← public API
│           ├── eval.rs         ← AST + evaluator + formatters
│           ├── parser.rs       ← tokenizer + recursive-descent parser
│           └── memory.rs       ← memory cells + persistence
└── docs/                       ← this mdBook
```

## Data flow

```
User input (String)
    │
    ▼
memory::expand_memory_refs()    ← replace m1..m9 refs with values
    │
    ▼
parser::parse(input, acc) → Expr
    │                           ← recursive-descent parser
    │                             produces an AST
    ▼
eval::eval(&Expr) → f64
    │
    ▼
eval::format_value(n, precision, base) → String
    │
    ▼
stdout
```

## Module responsibilities

| Module | Responsibility |
|---|---|
| `main.rs` | Parse CLI args, detect stdin mode, dispatch |
| `repl.rs` | REPL event loop, pipe line-reader, shared `evaluate()`, display logic |
| `help.rs` | Static help string, interpolates `CARGO_PKG_VERSION` |
| `eval.rs` | `Expr` AST, `Op`, `Base`; `eval()`, `format_value()`, `format_number()` |
| `parser.rs` | Tokenizer, recursive-descent parser, `parse()`, `is_partial()` |
| `memory.rs` | `Memory` struct, directive parser, ref expander, file I/O |

## Dependency graph

```
ccalc (binary)
  ├── ccalc-engine (local)
  │     └── dirs
  └── rustyline
```

## Design principles

- **No runtime allocations on the hot path** beyond the input string itself.
- **`crate::eval` is dependency-free** — no I/O, no allocations beyond `String` errors.
- **Version is defined once** in `[workspace.package]`; both crates inherit it.
- The engine crate has no knowledge of the terminal or REPL — it only deals with
  strings and numbers.
