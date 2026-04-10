% control_flow.calc — Phase 11 (if/elseif/else, for, while, break, continue,
%                                 compound assignment)
%
% Topic: classic number-theory exercises as control-flow showcases.
%        All expected results are annotated so the output can be verified
%        at a glance.
%
% Usage: ccalc control_flow.calc

% ── 1. if / elseif / else ────────────────────────────────────────────────────

fprintf('=== 1. Grade classifier ===\n')

score = 73;

if score >= 90
  grade = 'A';
elseif score >= 80
  grade = 'B';
elseif score >= 70
  grade = 'C';
elseif score >= 60
  grade = 'D';
else
  grade = 'F';
end

fprintf('score %d  ->  grade %s\n', score, grade)

% ── 2. for + compound assignment — sum of squares ────────────────────────────

fprintf('\n=== 2. Sum of squares 1..10 ===\n')

total = 0;
for k = 1:10
  total += k ^ 2;
end

fprintf('sum(k^2, k=1..10) = %d   (exact: 385)\n', total)

% ── 3. for + continue — sum only odd numbers ─────────────────────────────────

fprintf('\n=== 3. Sum of odd numbers in 1..20 ===\n')

s = 0;
for n = 1:20
  if mod(n, 2) == 0
    continue
  end
  s += n;
end

fprintf('sum of odds in 1..20 = %d   (exact: 100)\n', s)

% ── 4. for + break — first multiple of 7 above 50 ────────────────────────────

fprintf('\n=== 4. First multiple of 7 above 50 ===\n')

result = 0;
for n = 51:200
  if mod(n, 7) == 0
    result = n;
    break
  end
end

fprintf('first multiple of 7 above 50: %d   (exact: 56)\n', result)

% ── 5. for — count primes up to 30 (trial division) ──────────────────────────

fprintf('\n=== 5. Primes up to 30 ===\n')

count = 0;
for p = 2:30
  is_prime = 1;
  for d = 2:floor(sqrt(p))
    if mod(p, d) == 0
      is_prime = 0;
      break
    end
  end
  if is_prime
    count++;
    fprintf('  %d\n', p)
  end
end

fprintf('total: %d primes   (exact: 10)\n', count)

% ── 6. while — Newton-Raphson square root ────────────────────────────────────

fprintf('\n=== 6. Newton-Raphson: sqrt(2) ===\n')

x = 1.0;
iters = 0;
while abs(x ^ 2 - 2) > 1e-12
  x = (x + 2 / x) / 2;
  iters++;
end

fprintf('sqrt(2) approx  = %.15f\n', x)
fprintf('sqrt(2) builtin = %.15f\n', sqrt(2))
fprintf('converged in %d iterations\n', iters)

% ── 7. while + break/continue — Collatz sequence ─────────────────────────────

fprintf('\n=== 7. Collatz sequence from 27 ===\n')

n = 27;
steps = 0;
peak = 27;

while n ~= 1
  if mod(n, 2) == 0
    n /= 2;
  else
    n = 3 * n + 1;
  end
  steps++;
  if n > peak
    peak = n;
  end
end

fprintf('steps to reach 1: %d   (exact: 111)\n', steps)
fprintf('peak value:       %d   (exact: 9232)\n', peak)
