# Phase 15.6 — Variable Scoping

**Version:** v0.21.0+006–010  
**Status:** Complete

## Overview

Phase 15.6 adds MATLAB/Octave-compatible variable scoping: `global` and
`persistent` variables, `private/` directory isolation, and package namespaces
(`+pkg/`). These mechanisms were implemented together because they form a
coherent scoping hierarchy.

## `global` variables

`global x` declares `x` as a variable shared across all functions (and the
base workspace) that also declare `global x`. The implementation uses a
thread-local `GLOBAL_STORE: RefCell<HashMap<String, Value>>` in `eval.rs`
with a `GLOBAL_NAMES_STACK` that tracks which names each function frame has
declared as global.

Key functions in `eval.rs`:
- `global_declare(name)` — adds `name` to the current frame's set
- `global_set(name, val)` / `global_get(name)` — read/write the shared store
- `global_frame_push()` / `global_frame_pop()` — manage per-call frames
- `global_refresh_into_env(env)` — copies current globals into local env on call entry

## `persistent` variables

`persistent x` keeps a per-function value between calls. The implementation
uses `PERSISTENT_STORE: RefCell<HashMap<(String, String), Value>>` keyed by
`(function_name, variable_name)`.

**Write-through semantics (critical for memoization):** when `Stmt::Assign`
or `Stmt::IndexSet` targets a persistent variable, the new value is written to
`PERSISTENT_STORE` immediately — not only when the function returns. This
ensures recursive calls see the updated value:

```matlab
% Without write-through, fib_memo(n) would be O(2^n) because recursive calls
% see a stale copy of cache. With write-through, it is O(n).
cache(n) = fib_memo(n-1) + fib_memo(n-2);   % written through immediately
```

For `IndexSet`, the implementation also refreshes from the store before
applying the partial update, so recursive writes are not overwritten by a
stale parent frame.

## `private/` directory scoping

Functions in a `private/` sub-directory are visible only to scripts in the
parent directory. Two changes enforce this:

1. `collect_dirs_recursive` in `config.rs` skips directories named `private`
   so they are never added to the session search path.
2. `resolve_script_path` in `exec.rs` only prepends `dir/private/` to the
   search when `dir` comes from `SCRIPT_DIR_STACK` (the calling script's own
   directory), never from `SESSION_PATH` or CWD.

## Output suppression fix (`silence_all`)

Function bodies must suppress all output. The pre-existing silencing only
covered top-level statements; nested bodies inside `if`/`for`/`while`/`switch`
still printed. The new `silence_all(stmts)` function in `exec.rs` recursively
walks the full statement tree and sets every `(Stmt, bool)` to `(stmt, true)`.

## Single-line block fix inside function bodies

The REPL's single-line block bypass (`is_single_line_block` detection) was
executing `if cond; body; end` immediately even when `block_depth > 0` (inside
a buffered function definition). The fix adds a `block_depth == 0` guard so
single-line blocks inside a function body are appended to the buffer rather
than executed at the top level.

## Tests

`cargo test` — all 667 tests pass.

## Example

```bash
ccalc examples/scoping/scoping.calc
```
