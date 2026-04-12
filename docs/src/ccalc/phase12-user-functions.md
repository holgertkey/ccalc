# Phase 12 — User-defined Functions

**Version:** v0.17.0  
**Prerequisite:** Phase 11.5 (extended control flow, `return`, `run`/`source`)

---

## Overview

Phase 12 introduces user-defined named functions, multiple return values,
the `return` statement for early exit, and anonymous functions (lambdas) created
with `@(params) expr`.

---

## Named functions

```matlab
function result = name(p1, p2)
  ...
  result = expr;
end
```

Functions are defined at the top level — in the REPL or in a script file.
They are stored in the workspace like any variable and persist until `clear` is called.

### Single return value

```matlab
function y = square(x)
  y = x ^ 2;
end

square(5)     % 25
square(12)    % 144
```

### Multiple return values

```matlab
function [mn, mx, avg] = stats(v)
  mn  = min(v);
  mx  = max(v);
  avg = mean(v);
end

data = [4 7 2 9 1 5 8 3 6];
[lo, hi, mu] = stats(data);
% lo = 1  hi = 9  mu = 5
```

### Discarding outputs with `~`

```matlab
[~, top, ~] = stats([10 30 20]);   % top = 30
```

---

## `nargin` — optional arguments

`nargin` is injected into every function body and holds the number of arguments
actually passed by the caller. Use it to implement default parameter values:

```matlab
function y = power_fn(base, exp)
  if nargin < 2
    exp = 2;
  end
  y = base ^ exp;
end

power_fn(5)     % 25   (exp defaults to 2)
power_fn(2, 8)  % 256
power_fn(3, 3)  % 27
```

---

## `return` — early exit

`return` exits the function immediately. All output variables must be assigned
before `return` is reached:

```matlab
function g = gcd_fn(a, b)
  while b ~= 0
    r = mod(a, b);
    a = b;
    b = r;
  end
  g = a;
end
```

Recursive early return:

```matlab
function result = factorial_r(n)
  if n <= 1
    result = 1;
    return
  end
  result = n * factorial_r(n - 1);
end

factorial_r(7)   % 5040
```

---

## Scope

Each function call creates a **fresh isolated scope**:

- The caller's data variables (scalars, matrices, strings, etc.) are **not** visible.
- Declared parameters are bound in the local scope.
- `i`, `j`, and `ans` are pre-seeded.
- `nargin` and `nargout` are injected.
- All `Function` and `Lambda` values from the caller's workspace are forwarded,
  enabling self-recursion and mutual recursion without leaking data.

---

## Anonymous functions (lambdas)

```matlab
f = @(params) expr
```

`@(x) expr` creates a closure. The current environment is **captured at
definition time** (lexical scoping):

```matlab
sq   = @(x) x ^ 2;
cube = @(x) x ^ 3;
hyp  = @(a, b) sqrt(a^2 + b^2);

sq(7)        % 49
cube(4)      % 64
hyp(3, 4)    % 5
```

### Lexical capture

Changing a variable after a lambda is defined does not affect the lambda:

```matlab
rate = 0.05;
interest = @(principal, years) principal * (1 + rate) ^ years;

interest(1000, 10)   % 1628.89  (captured rate = 0.05)

rate = 0.99;         % lambda is unaffected
interest(1000, 10)   % still 1628.89
```

---

## Lambdas as arguments

Pass a lambda to a named function using `@`:

```matlab
function s = midpoint(f, a, b, n)
  h = (b - a) / n;
  s = 0;
  for k = 1:n
    xm = a + (k - 0.5) * h;
    s += f(xm);
  end
  s *= h;
end

% integral of x^2 from 0 to 1 = 1/3
midpoint(@(x) x^2, 0, 1, 1000)         % 0.333333

% integral of sin(x) from 0 to pi = 2
midpoint(@(x) sin(x), 0, pi, 1000)     % 2.000001
```

---

## Functions returning functions

Named functions can return lambdas (higher-order programming):

```matlab
function f = make_adder(c)
  f = @(x) x + c;
end

add5  = make_adder(5);
add10 = make_adder(10);

add5(3)            % 8
add10(7)           % 17
add5(add10(1))     % 16
```

---

## Implementation details

| Concern | Solution |
|---|---|
| Circular dependency (`eval.rs` ↔ `parser.rs`) | Named functions store `body_source: String`; re-parsed on each call in `exec.rs` |
| Cross-module dispatch | Thread-local `FnCallHook` in `eval.rs`, registered by `exec::init()` |
| Lexical closure | Lambda captures `Env` clone at `@` parse time; stored as `Value::Lambda(Rc<dyn Fn>)` |
| Recursion | `call_user_function` copies all `Function`/`Lambda` entries from caller's env into local scope |
| Multi-return | `Value::Tuple(Vec<Value>)` returned and destructured by `Stmt::MultiAssign` |
| Empty call `f()` | Parser injects `Expr::Var("ans")`; both call sites trim 1 extra arg silently |

### New AST nodes and tokens

| Name | Kind | Description |
|---|---|---|
| `Stmt::FunctionDef` | Statement | `function [outs] = name(params) body end` |
| `Stmt::Return` | Statement | `return` inside a function |
| `Stmt::MultiAssign` | Statement | `[a, b] = expr` destructuring |
| `Expr::Lambda` | Expression | `@(params) expr` |
| `Token::At` | Token | `@` prefix for lambdas |
| `Signal::Return` | Signal | propagates early return through `exec_stmts` |

### New `Value` variants

| Variant | Description |
|---|---|
| `Value::Function { outputs, params, body_source }` | Named user-defined function |
| `Value::Lambda(LambdaFn)` | Anonymous function / closure |
| `Value::Tuple(Vec<Value>)` | Internal multi-return value |

---

## Example

Run the full demo:

```bash
ccalc examples/user_functions.calc
```
