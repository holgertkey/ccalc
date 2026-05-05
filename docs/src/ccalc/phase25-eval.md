# Phase 25 — Dynamic Evaluation & Timing

**Trigger:** `eval()` is used in parameter-sweep scripts, metaprogramming
patterns, and anywhere variable names are constructed at runtime. `tic`/`toc`
appear in virtually every performance-sensitive script.

**Prerequisite:** Phase 12 (full evaluator pipeline — needed for the recursive
`exec_stmts` call inside `eval`); Phase 11.5 (`RUN_DEPTH` recursion guard).

---

## 25a — `eval` — string execution

`eval(str)` executes a string as code in the current workspace. Variable
mutations persist in the caller's scope, matching MATLAB/Octave semantics.

`eval(str, catch_str)` is the two-argument form: if `str` raises an error,
`catch_str` is executed and the original error is suppressed (stored in
`lasterr()`).

### Implementation

- **Statement context** (`eval(...)` as a standalone statement): intercepted in
  `exec_stmts` inside `exec.rs`, just after the `run`/`source` intercept.
  Uses the same `RUN_DEPTH` thread-local (max 64) to prevent infinite
  recursion. Calls `parse_stmts` → `exec_stmts` with the **caller's** `env`
  and `io`, so all mutations persist.

- **Expression context** (`y = eval('...')`): falls through to `call_builtin`
  in `eval.rs`. Uses `EVAL_STR_HOOK` (registered by `exec::init()`) which
  clones `env`, runs `exec_stmts` against the clone, and returns `ans`.
  Variable mutations inside the string are **discarded** — only `ans` is
  returned.

- `EVAL_STR_HOOK` follows the same hook pattern as `FN_CALL_HOOK` to avoid a
  circular dependency between `eval.rs` and `exec.rs`.

---

## 25b — `tic` / `toc` — elapsed time

`tic` stores `Instant::now()` in a thread-local `TIC_TIME`. `toc` reads the
elapsed duration and returns it as a `Scalar` in seconds. Multiple `toc` calls
after a single `tic` are valid; the timer is not reset by `toc`.

Both names are added to the `no_ans_inject` list so that `tic()` / `toc()`
called with empty parentheses do not have `ans` injected as an argument.

The `Expr::Var` handler in `eval_inner` is extended to fall back to
`call_builtin(name, &[], ...)` when a name is not found in the environment,
so that bare `tic` and `toc` (without parentheses) are recognized as
zero-argument function calls.

---

## Files changed

| File | Change |
|------|--------|
| `crates/ccalc-engine/src/eval.rs` | `EVAL_STR_HOOK` + `TIC_TIME` thread-locals; `tic`/`toc`/`eval` in `call_builtin`; zero-arg fallback in `Expr::Var`; `tic`/`toc`/`eval` added to `builtin_names()` and `no_ans_inject` |
| `crates/ccalc-engine/src/exec.rs` | `eval_str_impl`; `set_eval_str_hook` registered in `init()`; `eval` intercept in `Stmt::Expr` |
| `crates/ccalc-engine/src/eval_tests.rs` | `mod phase25_tests` — 11 tests |
| `crates/ccalc/src/help.rs` | `print_eval()` topic |
| `docs/src/guide/eval.md` | User guide page |
| `docs/src/SUMMARY.md` | Added entries |

**Test count:** 866 total (11 new in `phase25_tests`).
