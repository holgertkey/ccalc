# Phase 14 — Error Handling

**Version:** 0.20.0+002
**Prerequisite:** Phase 13 (structs — for `catch e` with `e.message`).

Scripts can now raise, catch, and recover from runtime errors without crashing the session. Two complementary mechanisms are provided: MATLAB-compatible `try/catch` block syntax and functional forms (`pcall`, `try(expr, default)`) as idiomatic ccalc alternatives.

---

## 14a — `error` and `warning`

### `error(fmt, args...)`

Raises a runtime error with a printf-formatted message. Execution of the current block stops immediately; the error propagates to the nearest enclosing `try/catch` or to the REPL prompt.

```matlab
error('value must be positive')
error('expected %d arguments, got %d', 2, nargin)
error('singular matrix detected at step %d', k)
```

### `warning(fmt, args...)`

Prints a warning message to stderr and continues execution normally.

```matlab
warning('result may be inaccurate')
warning('condition number = %.1e exceeds threshold', cond(A))
```

Both functions use the same printf format specifiers as `fprintf` and `sprintf` (`%d`, `%f`, `%g`, `%s`, `%e`, `%%`, width/precision flags).

---

## 14b — `lasterr`

`lasterr` stores the message from the most recent runtime error, whether caught by `try/catch` or displayed at the REPL prompt.

```matlab
lasterr()         % return last error message
lasterr('')       % clear (returns previous value)
lasterr(msg)      % set message; returns previous value
```

Example:

```matlab
lasterr('');
try
  inv([1 0; 0 0])
catch
end
msg = lasterr()    % 'singular matrix'
```

---

## 14c — `try/catch/end`

MATLAB-compatible protected block. If any statement in the `try` body raises an error, execution jumps immediately to the `catch` body; remaining `try` statements are skipped. `lasterr` is set on entry to the catch body.

### Anonymous catch

```matlab
try
  risky_code()
catch
  fallback_code()
end
```

### Named catch

`catch e` binds a struct with field `message` to the catch variable:

```matlab
try
  result = risky_function(data)
catch e
  fprintf('caught: %s\n', e.message)
  result = default_value
end
```

### try with no catch

Silently swallows any error from the try body:

```matlab
try
  might_fail()
end
```

### Nesting

`try/catch` blocks may be nested to any depth. An error re-raised from a catch body propagates to the next outer handler:

```matlab
try
  try
    error('inner')
  catch e
    fprintf('inner caught: %s\n', e.message)
    error('re-raised: %s', e.message)
  end
catch e
  fprintf('outer caught: %s\n', e.message)
end
```

### In loops

`break`, `continue`, and `return` work normally inside `try` and `catch` bodies:

```matlab
for k = 1:numel(data)
  try
    result = process(data(k))
  catch e
    fprintf('step %d failed: %s\n', k, e.message)
    continue
  end
  fprintf('step %d: %g\n', k, result)
end
```

---

## 14d — `try(expr, default)`

Inline functional fallback. Evaluates `expr`; returns its value on success. If `expr` raises an error, evaluates and returns `default` instead. The default expression is **not** evaluated unless `expr` fails (lazy semantics).

```matlab
x = try(inv(A), eye(n))           % fallback to identity if singular
n = try(str2num(s), 0)            % fallback to 0 if s is not a number
v = try(risky(data), NaN)         % NaN sentinel on error
```

This is a **special form** — `try(expr, default)` looks like a function call but its arguments are not pre-evaluated.

---

## 14e — `pcall`

Protected call: invoke any callable and capture success/failure as a value. Composable with `if`, multi-assign, and loops.

```matlab
[ok, val] = pcall(@func, arg1, arg2, ...)
```

Return values:
- **Success:** `ok = 1`, `val = ` function return value
- **Failure:** `ok = 0`, `val = ` error message string

```matlab
[ok, x] = pcall(@inv, A)
if ~ok
  fprintf('inv failed: %s\n', x)
  x = eye(n)
end

[ok, y] = pcall(@(x) sqrt(x), -1)   % ok=0, y='sqrt of negative'
```

`pcall` is particularly useful in loops where you want to continue processing after a failed step:

```matlab
for k = 1:numel(data)
  [ok, v] = pcall(@process, data(k))
  if ok
    results(k) = v
  else
    fprintf('step %d: %s\n', k, v)
    results(k) = 0
  end
end
```

---

## Note on `e` as a variable

The constant `e` (Euler's number, ≈ 2.718) and a catch variable named `e` do not conflict. Variable assignments always shadow built-in constants:

```matlab
try
  error('oops')
catch e
  fprintf('message: %s\n', e.message)   % e is a struct here
end
% After the block, 'e' is no longer in scope (try/catch does not leak)
```

---

## Summary

| Feature | Description |
|---------|-------------|
| `error(fmt, args...)` | Raise a runtime error |
| `warning(fmt, args...)` | Print warning, continue |
| `lasterr()` | Get last error message |
| `lasterr(msg)` | Set last error message |
| `try / catch / end` | Anonymous protected block |
| `try / catch e / end` | Named: `e.message` = error string |
| `try(expr, default)` | Inline fallback (lazy) |
| `pcall(@f, args...)` | Protected call → `[ok, val]` |

See also: [Phase 13 — Structs](./phase13-structs.md) · [Phase 11 — Control Flow](./phase11-control-flow.md)

Example file: `ccalc examples/error_handling.calc`
