# Phase 13.6 — Backslash Operator and Path System

**Version:** 0.20.0

Two independent features that fill gaps left by earlier phases: the
backslash left-division operator (`\`) for linear system solving, and a
session search path (`addpath` / `rmpath` / `path()`) for script lookup.

---

## 13.6a — Backslash operator `\`

### What it does

`A \ b` solves the linear system `A * x = b` without computing `inv(A)`
explicitly. This is the standard MATLAB idiom: more numerically stable and
more concise than `inv(A) * b`.

```matlab
A = [2 1; 5 7];
b = [11; 13];

x1 = inv(A) * b;      % explicit inverse — less stable
x2 = A \ b;           % left division — preferred

fprintf('x via inv: '); disp(x1')
fprintf('x via \\:   '); disp(x2')
```

### Scalar form

For scalars `a \ b` is equivalent to `b / a`:

```matlab
4 \ 20          % → 5     (same as 20 / 4)
3 \ [6; 9; 12]  % → [2; 3; 4]   (scalar divides into each element)
```

### Multiple right-hand sides

When `b` is a matrix, each column is solved independently — equivalent to
`[A\b1, A\b2, ...]` in a single operation:

```matlab
C  = [4 1; 2 3];
B2 = [5 1; 10 0];   % two right-hand sides
X  = C \ B2;         % solve both columns at once
disp(C * X - B2)     % residual should be ~0
```

### Precedence

`\` has the same precedence as `*` and `/`, evaluated left to right:

```matlab
2 \ 8 / 2   % → (8/2) / 2 = 2   (same level, left-to-right)
```

### Implementation

- **Token:** `Token::Backslash`
- **AST:** `Op::LDiv`
- **Evaluator cases:**
  - `Scalar \ Scalar` → `b / a` (error if `a == 0`)
  - `Matrix \ Matrix` → Gaussian elimination with partial pivoting (augmented matrix)
  - `Scalar \ Matrix` → divide each element by the scalar
  - `Matrix \ Scalar` → solve `A * x = [s; s; ...]` (treated as 1-column RHS)

---

## 13.6b — Session search path

### What it does

The session search path controls where `run()` and `source()` look for
script files. Without it, scripts must live in the current working directory.

**Search order:**
1. Current working directory (always first)
2. Session path entries, in order

### Commands

```matlab
addpath('/my/scripts')            % prepend — highest priority
addpath('/my/utils', '-end')      % append  — lowest priority
rmpath('/my/scripts')             % remove an entry
path()                            % display all entries
```

Duplicate entries are silently deduplicated. Adding an existing path moves
it to the front (or keeps it at the end with `-end`). `~` is expanded to the
user's home directory on all platforms.

### Persistence

`addpath` / `rmpath` affect the **current session only**. To make a path
permanent, add it to `~/.config/ccalc/config.toml`:

```toml
path = [
  "~/.config/ccalc/lib",
  "/home/user/scripts",
]
```

Config paths are loaded at startup before any session `addpath` calls.

### Example session

```matlab
addpath('/tmp/mylib');
addpath('/tmp/utils', '-end');
path()             % /tmp/mylib  /tmp/utils

addpath('/tmp/mylib');   % duplicate → moved to front, no second copy
path()

rmpath('/tmp/utils');
path()             % /tmp/mylib only
```

### Implementation

- `SESSION_PATH: RefCell<Vec<PathBuf>>` thread-local in `exec.rs`
- `session_path_init` / `session_path_add` / `session_path_remove` / `session_path_list` — public functions
- `resolve_script_path` checks `SESSION_PATH` entries after the CWD
- Config `path = [...]` array: `#[serde(default)]` field in `Config`, loaded at startup via `session_path_init(cfg.search_path())`
- `addpath` / `rmpath` / `path()` intercepted in `exec_stmts` (block mode) and `try_path_cmd()` in `repl.rs` (pipe / single-line REPL mode)

---

## Files changed

| File | Change |
|------|--------|
| `ccalc-engine/src/parser.rs` | `Token::Backslash`, `parse_term` case for `\`, `split_stmts` `''`-escape fix |
| `ccalc-engine/src/eval.rs` | `Op::LDiv`, `solve_linear()`, `eval_binop` cases |
| `ccalc-engine/src/exec.rs` | `SESSION_PATH`, path functions, `addpath`/`rmpath`/`path()` intercept |
| `ccalc/src/config.rs` | `path: Vec<String>`, `search_path()`, `expand_tilde()` |
| `ccalc/src/repl.rs` | `session_path_init` calls, `try_path_cmd()` |
| `examples/matrix_ops.calc` | Updated to demo `A\b` and multiple RHS |
| `examples/path_system.calc` | New file — full path system demo |

---

## Examples

```bash
ccalc examples/matrix_ops.calc     # linear solve, element-wise, det/inv
ccalc examples/path_system.calc    # addpath/rmpath/path() demo
```
