# Dynamic Evaluation & Timing

## eval — string execution

`eval(str)` executes a string as ccalc code in the **current workspace**.
Variables defined inside the string persist in the caller's scope, matching
MATLAB/Octave semantics.

```matlab
eval('x = sqrt(2)')    % x is now defined in the workspace
x                      % → 1.4142…

eval('disp(pi)')       % prints 3.14159…
```

### Dynamic variable naming

A common idiom is building variable names at runtime with `sprintf`:

```matlab
for k = 1:3
  eval(sprintf('v%d = k*k', k))
end
v1    % → 1
v2    % → 4
v3    % → 9
```

### Two-argument form — catching errors

`eval(try_str, catch_str)` executes `catch_str` if `try_str` raises an error.
The original error message is available via `lasterr()` inside the catch string.

```matlab
eval('error(''oops'')', 'fprintf(''caught: %s\n'', lasterr())')
```

### eval in expression context

When `eval` is used on the right-hand side of an assignment, it returns `ans`
from the inner execution. Variable mutations inside do **not** propagate back
to the caller's workspace.

```matlab
y = eval('2 + 2')    % y = 4
```

### Nesting

`eval` calls can be nested. The depth limit is 64 (shared with `run`/`source`).

---

## tic / toc — elapsed time

`tic` starts (or restarts) a timer. `toc` reads the elapsed time in seconds
since the last `tic`.

```matlab
tic
A = rand(500) * rand(500);
t = toc                           % → e.g. 0.0042  (seconds)

tic
for k = 1:1000
  x = k^2;
end
fprintf('loop: %.4f s\n', toc)
```

Both `tic` and `toc` can be written with or without parentheses:

```matlab
tic
t = toc
% same as
tic()
t = toc()
```

Multiple `toc` calls after a single `tic` are valid — the timer is not reset
by `toc`. Calling `toc` before any `tic` is an error.

---

## See also

- [`help eval`](../ccalc/phase25-eval.md) — reference page
- [`help control`](control-flow.md) — control flow
- [`help errors`](error-handling.md) — error handling
