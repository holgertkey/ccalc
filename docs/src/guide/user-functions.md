# User-defined Functions

ccalc supports user-defined named functions, multiple return values, and
anonymous functions (lambdas) using Octave/MATLAB syntax.

## Named functions

```matlab
function result = name(p1, p2)
  ...
  result = expr;
end
```

Define a function at the top level in the REPL or in a `.calc` / `.m` script
file. Once defined, the function is stored in the workspace and can be called
like any built-in.

### Single return value

```matlab
function y = square(x)
  y = x ^ 2;
end

square(5)     % 25
```

### Multiple return values

```matlab
function [mn, mx, avg] = stats(v)
  mn  = min(v);
  mx  = max(v);
  avg = mean(v);
end

[lo, hi, mu] = stats([4 7 2 9 1 5 8 3 6]);
% lo = 1   hi = 9   mu = 5
```

### Discarding outputs

Use `~` in the assignment target to ignore individual outputs:

```matlab
[~, top, ~] = stats([10 30 20]);   % top = 30
```

---

## `nargin` — optional parameters

`nargin` holds the number of arguments actually passed:

```matlab
function y = power_fn(base, exp)
  if nargin < 2
    exp = 2;   % default exponent
  end
  y = base ^ exp;
end

power_fn(5)     % 25   (exp = 2 by default)
power_fn(2, 8)  % 256
```

---

## `return` — early exit

```matlab
function result = factorial_r(n)
  if n <= 1
    result = 1;
    return      % exit immediately — no further code runs
  end
  result = n * factorial_r(n - 1);
end

factorial_r(7)   % 5040
```

---

## Scope

Each call gets its own isolated scope:

- The caller's data variables are **not** visible inside the function.
- Parameters are bound locally.
- Other functions and lambdas from the caller's workspace **are** forwarded,
  enabling self-recursion and mutual recursion.

```matlab
function g = gcd_fn(a, b)
  while b ~= 0
    r = mod(a, b);
    a = b;
    b = r;
  end
  g = a;
end

gcd_fn(252, 105)   % 21
```

---

## Anonymous functions

`@(params) expr` creates an anonymous function (lambda):

```matlab
sq  = @(x) x ^ 2;
hyp = @(a, b) sqrt(a^2 + b^2);

sq(7)       % 49
hyp(3, 4)   % 5
```

### Lexical capture

A lambda captures the value of free variables **at definition time**:

```matlab
rate = 0.05;
interest = @(p, n) p * (1 + rate) ^ n;

interest(1000, 10)   % 1628.89  (uses captured rate = 0.05)

rate = 0.99;         % does not affect the already-created lambda
interest(1000, 10)   % still 1628.89
```

---

## Passing functions as arguments

Use `@name` to pass an existing function, or `@(x) expr` inline:

```matlab
function s = midpoint(f, a, b, n)
  h = (b - a) / n;
  s = 0;
  for k = 1:n
    s += f(a + (k - 0.5) * h);
  end
  s *= h;
end

midpoint(@(x) x^2,    0, 1, 1000)    % ≈ 0.333333  (∫₀¹ x² dx)
midpoint(@(x) sin(x), 0, pi, 1000)   % ≈ 2.000001  (∫₀ᵖⁱ sin x dx)
```

---

## Functions returning functions

```matlab
function f = make_adder(c)
  f = @(x) x + c;
end

add5  = make_adder(5);
add10 = make_adder(10);

add5(3)         % 8
add10(7)        % 17
add5(add10(1))  % 16
```

---

## Full example

```bash
ccalc examples/user_functions.calc
```

See also: [`help userfuncs`](../ccalc/phase12-user-functions.md) for the
in-REPL reference, and [Control Flow](control-flow.md) for `if`, `for`,
`while`, `break`, and `return`.
