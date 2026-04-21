# Variable Scoping

ccalc provides four mechanisms to control visibility and lifetime of variables
across function calls and files.

## `global` — shared workspace storage

Declare the **same name** in every function that needs to share it. Changes in
one function are immediately visible in all others and in the base workspace.

```matlab
function reset_counter()
  global g_count
  g_count = 0;
end

function increment(step)
  global g_count
  g_count = g_count + step;
end

function n = read_counter()
  global g_count
  n = g_count;
end

reset_counter()
increment(1)
increment(1)
increment(1)
read_counter()   % 3
```

**Typical use cases:** configuration objects, counters, accumulators shared by
multiple functions without threading the value through every argument list.

---

## `persistent` — per-function long-lived storage

A persistent variable keeps its value **between calls to the same function**.
On the very first call the variable is `[]`; use `isempty()` to initialise it.

```matlab
function n = how_many_calls()
  persistent call_count
  if isempty(call_count)
    call_count = 0;
  end
  call_count += 1;
  n = call_count;
end

how_many_calls()   % 1
how_many_calls()   % 2
how_many_calls()   % 3
```

### Memoization

Persistent variables are ideal for caching computed results. The write-through
semantics ensure that recursive calls see each other's cache updates immediately:

```matlab
function f = fib_memo(n)
  persistent cache
  if isempty(cache)
    cache = zeros(1, 100);
    cache(1) = 1;  cache(2) = 1;
  end
  if cache(n) ~= 0
    f = cache(n);
    return
  end
  cache(n) = fib_memo(n-1) + fib_memo(n-2);
  f = cache(n);
end

fib_memo(30)   % 832040  (computed in O(n) time, not O(2^n))
```

**Contrast with `global`:** a persistent variable is private to its function —
no other function can read or write it. A global variable is shared by any
function that declares it.

---

## `private/` — directory-scoped helpers

Functions placed in a `private/` sub-directory are visible **only** to scripts
and functions in the parent directory. Any other caller receives an "Unknown
function" error.

```
mylib/
  normalize.calc     ← can call clamp() and lerp()
  private/
    clamp.calc       ← invisible outside mylib/
    lerp.calc        ← invisible outside mylib/
```

```matlab
% normalize.calc — parent can call private helpers directly
function y = normalize(data, lo, hi)
  span = hi - lo;
  for k = 1:numel(data)
    y(k) = lerp(0, 1, (clamp(data(k), lo, hi) - lo) / span);
  end
end
```

`private/` directories are **not** added to the session search path even when a
parent directory is included in `config.toml` or via `addpath`. The privacy
boundary is enforced by the file-system layout, not by any configuration.

---

## Packages (`+pkg/`) — named namespaces

A directory whose name starts with `+` is a **package**. Functions inside are
invisible at the top level and must be called with the package prefix:

```matlab
pkg.function(args)
```

### Layout

```
+utils/
  clamp.calc          % utils.clamp(x, lo, hi)
  lerp.calc           % utils.lerp(a, b, t)
+geom/
  circle_area.calc    % geom.circle_area(r)
  rect_area.calc      % geom.rect_area(w, h)
```

### Usage

```matlab
utils.clamp(-3, 0, 10)       % 0
utils.clamp( 5, 0, 10)       % 5
utils.lerp(0, 100, 0.25)     % 25

geom.circle_area(1)          % 3.14159...
geom.rect_area(4, 5)         % 20

% Packages compose naturally with each other and with regular expressions
x = utils.clamp(utils.lerp(-10, 20, 0.5), 0, 10);   % 5
```

### Nested packages

Sub-directories inside a package directory that also start with `+` form nested
packages:

```
+geom/
  +solid/
    sphere_vol.calc   % geom.solid.sphere_vol(r)
```

```matlab
geom.solid.sphere_vol(3)   % 4/3 * pi * 27
```

### Autoload

Package functions are loaded on the first call. The search follows the standard
path order: calling script's directory → CWD → session path. No `source()`
call is needed.

---

## Summary

| Mechanism   | Visibility                             | Lifetime                      |
|-------------|----------------------------------------|-------------------------------|
| `global`    | Any function that declares it          | Until `clear` or session end  |
| `persistent`| Private to the declaring function      | Until session end             |
| `private/`  | Parent directory only                  | File exists on disk           |
| `+pkg/`     | Anyone, via `pkg.func()` syntax        | Autoloaded on first call      |

---

## Full example

```bash
ccalc examples/scoping/scoping.calc
```

See also: [`help scoping`](../../ccalc/phase156-scoping.md) for the in-REPL
reference, and [User-defined Functions](user-functions.md).
