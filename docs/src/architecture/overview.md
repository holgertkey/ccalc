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
│           ├── env.rs          ← Env type, workspace save/load
│           ├── eval.rs         ← AST + evaluator + formatters + Base enum
│           └── parser.rs       ← tokenizer + recursive-descent parser, Stmt enum
└── docs/                       ← this mdBook
```

## Data flow

```
User input (String)
    │
    ▼
parser::parse(input) → Stmt (Assign | Expr)
    │                       ← recursive-descent parser
    │                         produces an AST node
    ▼
eval::eval(&Expr, &Env) → f64   (Value enum from Phase 3)
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
| `main.rs` | Parse CLI args, detect stdin mode (REPL / pipe / file / arg), dispatch |
| `repl.rs` | REPL event loop, pipe line-reader, shared `evaluate()`, display logic |
| `help.rs` | Static help string |
| `env.rs` | `Env` type (`HashMap<String, f64>`), workspace save/load to disk |
| `eval.rs` | `Expr` AST, `Op`, `Base`; `eval()`, `format_value()`, `format_number()` |
| `parser.rs` | Tokenizer, recursive-descent parser, `parse()`, `is_partial()`, `Stmt` enum |

## Dependency graph

```
ccalc (binary)
  ├── ccalc-engine (local)
  │     └── dirs
  └── rustyline
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
