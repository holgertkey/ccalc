# Phase 10.5 — File I/O and Filesystem Queries

**Status:** Complete ✅  
**Version:** v0.14.0+006  
**Prerequisite:** Phase 10 (string types for filename arguments)

---

## What was implemented

### 10.5a — File handles

Low-level file I/O using integer file descriptors, matching the MATLAB/Octave `fopen`/`fclose` model.

| Function | Description |
|----------|-------------|
| `fopen(path, mode)` | Open file; returns fd (≥3) or -1 on failure |
| `fclose(fd)` | Close by fd; returns 0 on success, -1 on error |
| `fclose('all')` | Close all open file handles |
| `fgetl(fd)` | Read one line, strip trailing newline; returns -1 at EOF |
| `fgets(fd)` | Read one line, keep trailing newline; returns -1 at EOF |
| `fprintf(fd, fmt, ...)` | Write formatted output to file descriptor |

Supported modes: `'r'` read, `'w'` write (create/truncate), `'a'` append, `'r+'` read+write.

Pre-opened virtual descriptors: fd 1 = stdout, fd 2 = stderr (Octave convention).

### 10.5b — Delimiter-separated data

Read and write numeric data in CSV or TSV format.

| Function | Description |
|----------|-------------|
| `dlmread(path)` | Read; auto-detect `,` / `\t` / whitespace |
| `dlmread(path, delim)` | Explicit delimiter |
| `dlmwrite(path, A)` | Write matrix with `,` separator |
| `dlmwrite(path, A, delim)` | Explicit delimiter |

Returns `Value::Matrix`; all values must be numeric. Non-numeric data returns an error with the line number.

### 10.5c — Filesystem queries

| Function | Description |
|----------|-------------|
| `isfile(path)` | 1 if the path exists and is a regular file |
| `isfolder(path)` | 1 if the path exists and is a directory |
| `pwd()` | Current working directory as a char array |
| `exist(name)` | 1 if variable in workspace; 2 if a file exists on disk |
| `exist(name, 'var')` | Check workspace only |
| `exist(name, 'file')` | Check filesystem only; returns 2 (MATLAB numeric code) |

### 10.5d — Workspace with explicit path

Extension of the existing `ws`/`wl` commands to accept an explicit file path.

| Syntax | Description |
|--------|-------------|
| `save` | Save all variables to default path (alias for `ws`) |
| `save('path.mat')` | Save all variables to named file |
| `save('path.mat', 'x', 'y')` | Save specific variables only |
| `load` | Load from default path (alias for `wl`) |
| `load('path.mat')` | Load from named file |

Path argument can be a variable reference: `save(path_var)`.

Persisted types: `Scalar`, `Str` (char array), `StringObj`. Matrices, complex values, and `Void` are always skipped.

---

## Key implementation details

### `IoContext`

New struct in `ccalc-engine/src/io.rs`. Holds a `HashMap<i32, FileHandle>` and `next_fd: i32` (starts at 3). `FileHandle` is an enum: `Read(BufReader<File>)` | `Write(File)`. fd 1 and 2 are handled virtually in `write_to_fd`.

### `eval_with_io` / `eval`

```rust
pub fn eval(expr: &Expr, env: &Env) -> Result<Value, String>
pub fn eval_with_io(expr: &Expr, env: &Env, io: &mut IoContext) -> Result<Value, String>
```

Both are wrappers around private `eval_inner(expr, env, mut io: Option<&mut IoContext>)`. The `Option<&mut T>` pattern with `as_deref_mut()` reborrowing allows sequential mutable borrows in recursive calls without cloning.

### `call_builtin` signature change

```rust
fn call_builtin(name: &str, args: &[Value], env: &Env, io: Option<&mut IoContext>)
```

`env: &Env` was added for `exist('x', 'var')` which needs to inspect the workspace.

### `try_parse_save_load`

Intercepts `save`/`load` statements in `repl.rs` before they reach `evaluate()`. Accepts `env: &Env` for resolving variable-reference path arguments (`save(mat_path)` where `mat_path` is a `Str`/`StringObj` in the workspace).

```rust
enum SaveLoadCmd {
    Save { path: Option<String>, vars: Vec<String> },
    Load { path: Option<String> },
}
fn try_parse_save_load(stmt: &str, env: &Env) -> Option<SaveLoadCmd>
```

### Workspace serialization

```
name = 3.14          # Scalar
name = 'text'        # Str (char array)
name = "text"        # StringObj
```

Strings containing `'` or `"` characters (matching the quote style) or `\n` are skipped to preserve the line-based format.

---

## Tests

- `eval_tests.rs`: `test_fopen_write_and_fclose`, `test_fgetl_reads_lines`, `test_fgets_keeps_newline`, `test_fprintf_to_file`, `test_fclose_all`, `test_fopen_nonexistent_returns_minus_one`, `test_dlmwrite_and_dlmread_comma`, `test_dlmwrite_tab_delimiter`, `test_dlmread_empty_file`, `test_dlmread_non_numeric_error`, `test_dlmread_whitespace_auto`, `test_dlmwrite_scalar`, `test_isfile_existing_file`, `test_isfile_nonexistent`, `test_isfile_directory_is_false`, `test_isfolder_existing_dir`, `test_isfolder_file_is_false`, `test_exist_var_found`, `test_exist_var_not_found`, `test_exist_file_found`, `test_exist_file_not_found`, `test_exist_one_arg_checks_var_then_file`, `test_pwd_returns_string`, `test_fprintf_returns_void`
- `repl_tests.rs`: 9 new Phase 10.5d tests including `test_pipe_save_load_roundtrip` and `test_pipe_save_selective_vars`
- Total: 461 tests passing
