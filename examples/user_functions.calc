% user_functions.calc — Phase 12 (named functions, multiple return values,
%                                  anonymous functions / lambdas)
%
% Topic: classic mathematical and engineering problems solved with
%        user-defined functions and lambdas.
%        All expected results are annotated for easy verification.
%
% Usage: ccalc user_functions.calc

% ── 1. Basic named function — single return value ────────────────────────────
%
% Factorial via recursion.

function result = factorial_r(n)
  if n <= 1
    result = 1;
    return
  end
  result = n * factorial_r(n - 1);
end

fprintf('=== 1. Recursive factorial ===\n')
fprintf('  0! = %d   (exact: 1)\n',   factorial_r(0))
fprintf('  5! = %d   (exact: 120)\n', factorial_r(5))
fprintf('  7! = %d   (exact: 5040)\n', factorial_r(7))

% ── 2. Named function with early return ──────────────────────────────────────
%
% GCD via Euclidean algorithm — returns as soon as r == 0.

function g = gcd_fn(a, b)
  while b ~= 0
    r = mod(a, b);
    a = b;
    b = r;
  end
  g = a;
end

fprintf('\n=== 2. GCD (Euclidean) ===\n')
fprintf('  gcd(252, 105) = %d   (exact: 21)\n',  gcd_fn(252, 105))
fprintf('  gcd(360, 450) = %d   (exact: 90)\n',  gcd_fn(360, 450))
fprintf('  gcd(17,  13)  = %d    (exact: 1)\n',   gcd_fn(17, 13))

% ── 3. Named function — nargin for optional argument ─────────────────────────
%
% Power function: base ^ exp, default exponent = 2.

function y = power_fn(base, exp)
  if nargin < 2
    exp = 2;
  end
  y = base ^ exp;
end

fprintf('\n=== 3. power_fn — optional argument via nargin ===\n')
fprintf('  power_fn(5)    = %g   (exact: 25)\n',  power_fn(5))
fprintf('  power_fn(2, 8) = %g   (exact: 256)\n', power_fn(2, 8))
fprintf('  power_fn(3, 3) = %g   (exact: 27)\n',  power_fn(3, 3))

% ── 4. Multiple return values ─────────────────────────────────────────────────
%
% Statistics: min, max, mean and standard deviation in one call.

function [mn, mx, avg] = stats(v)
  mn  = min(v);
  mx  = max(v);
  avg = mean(v);
end

fprintf('\n=== 4. Multiple return values — stats ===\n')
data = [4 7 2 9 1 5 8 3 6];
[lo, hi, mu] = stats(data);
fprintf('  min  = %g   (exact: 1)\n', lo)
fprintf('  max  = %g   (exact: 9)\n', hi)
fprintf('  mean = %g   (exact: 5)\n', mu)

% ── 5. Tilde ~ to discard outputs ────────────────────────────────────────────

fprintf('\n=== 5. Discard outputs with ~ ===\n')
[~, top, ~] = stats([10 30 20]);
fprintf('  max of [10 30 20] = %g   (exact: 30)\n', top)

% ── 6. Anonymous functions (lambdas) — basics ────────────────────────────────

fprintf('\n=== 6. Lambda — basic arithmetic ===\n')

sq      = @(x) x ^ 2;
cube    = @(x) x ^ 3;
hyp     = @(a, b) sqrt(a^2 + b^2);
celsius = @(f) (f - 32) * 5 / 9;

fprintf('  sq(7)         = %g   (exact: 49)\n',    sq(7))
fprintf('  cube(4)       = %g   (exact: 64)\n',    cube(4))
fprintf('  hyp(3, 4)     = %g   (exact: 5)\n',     hyp(3, 4))
fprintf('  32 F in C     = %g   (exact: 0)\n',     celsius(32))
fprintf('  212 F in C    = %g   (exact: 100)\n',   celsius(212))

% ── 7. Lambda captures lexical environment ────────────────────────────────────
%
% The lambda closes over the value of 'rate' at definition time.
% Changing 'rate' afterwards does not affect the already-created lambda.

fprintf('\n=== 7. Lambda — lexical capture ===\n')

rate = 0.05;
interest = @(principal, years) principal * (1 + rate) ^ years;

fprintf('  5%% for 10 years on 1000: %g   (exact: 1628.89)\n', interest(1000, 10))

rate = 0.99;   % changing rate here does NOT affect the lambda above
fprintf('  rate changed but lambda uses captured 5%%: %g   (exact: 1628.89)\n', interest(1000, 10))

% ── 8. Lambda passed to a named function ─────────────────────────────────────
%
% Numerical integration via the midpoint rule — the integrand is a lambda.

function s = midpoint(f, a, b, n)
  h = (b - a) / n;
  s = 0;
  for k = 1:n
    xm = a + (k - 0.5) * h;
    s += f(xm);
  end
  s *= h;
end

fprintf('\n=== 8. Midpoint integration ===\n')

% integral of x^2 from 0 to 1 = 1/3
area1 = midpoint(@(x) x^2, 0, 1, 1000);
fprintf('  int(x^2, 0..1)      = %.6f   (exact: 0.333333)\n', area1)

% integral of sin(x) from 0 to pi = 2
area2 = midpoint(@(x) sin(x), 0, pi, 1000);
fprintf('  int(sin(x), 0..pi)  = %.6f   (exact: 2.000000)\n', area2)

% ── 9. Functions returning functions (higher-order) ───────────────────────────
%
% make_adder returns a lambda that adds a fixed constant.

function f = make_adder(c)
  f = @(x) x + c;
end

fprintf('\n=== 9. Higher-order functions — make_adder ===\n')

add5  = make_adder(5);
add10 = make_adder(10);

fprintf('  add5(3)   = %g   (exact: 8)\n',  add5(3))
fprintf('  add10(7)  = %g   (exact: 17)\n', add10(7))
fprintf('  add5(add10(1)) = %g   (exact: 16)\n', add5(add10(1)))

% ── 10. Fibonacci — iterative with named function ────────────────────────────

function f = fib(n)
  if n <= 0
    f = 0;
    return
  end
  if n == 1
    f = 1;
    return
  end
  a = 0;
  b = 1;
  for k = 2:n
    c = a + b;
    a = b;
    b = c;
  end
  f = b;
end

fprintf('\n=== 10. Fibonacci sequence ===\n')
for k = [0 1 5 10 15]
  fprintf('  fib(%2d) = %d\n', k, fib(k))
end
% Expected:
%   fib( 0) = 0
%   fib( 1) = 1
%   fib( 5) = 5
%   fib(10) = 55
%   fib(15) = 610

fprintf('\nDone.\n')
