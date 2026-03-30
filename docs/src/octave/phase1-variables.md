# Phase 1 — Variables and Assignment

**Status: ✅ Done** (v0.7.0)

**Goal**: `x = 5`, then `x + 1` → `6`

## What was implemented

### Variable environment (`Env`)

```rust
pub type Env = HashMap<String, f64>;
```

`eval` takes `&Env`. The REPL holds one `Env` per session, initialized with
`ans = 0.0` on startup.

### Assignment statement

`x = expr` is a top-level statement, separate from an expression:

```rust
pub enum Stmt {
    Assign(String, Expr),   // x = expr
    Expr(Expr),             // standalone expression → stored in ans
}
```

### Variable lookup in expressions

```rust
Expr::Var(String)
```

When the evaluator encounters `Var(name)`, it looks up `name` in `Env`.
Unknown names produce an error.

### `ans` — implicit last result

`ans` is the reserved name for the result of the last standalone expression.
It is initialized to `0.0` on startup and updated after every expression
that is not assigned to a named variable — exactly as in Octave/MATLAB.

Partial expressions (starting with an operator) automatically prepend `ans`:

```
[ 100 ]: / 4
[ 25 ]: + 5
[ 30 ]:
```

### Commands

| Command | Action |
|---|---|
| `who` | List all variables and their values |
| `clear` | Delete all variables (reinitializes `ans = 0`) |
| `clear x` | Delete variable `x` |
| `ws` | Save workspace to `~/.config/ccalc/workspace.toml` |
| `wl` | Load workspace from file |

### Workspace persistence

Variables are saved and loaded via `env::save_workspace_default()` /
`load_workspace_default()`. The file format is plain `name = value` lines.

## What was removed in this phase

- **`acc`** (old accumulator) — replaced by `ans`.
- **`m1`–`m9`** (memory cells) — replaced by named variables.
- **`c` command** — replaced by `clear`.

## Octave/MATLAB alignment

- `ans` follows Octave/MATLAB convention exactly.
- `pi` and `e` are resolved in the parser (not stored in `Env`).
- `%` terminates tokenization — everything after `%` on a line is a comment.
