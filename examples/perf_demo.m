% perf_demo.m — Phase 34b: bytecode VM performance demonstration
%
% This script demonstrates the speedup achieved by the bytecode compiler
% and register VM introduced in Phase 34b.
%
% Before 34b: nested matrix-fill loop (~25 s in release).
% After  34b: compiled to VM bytecode (<1 s in release).

fprintf('=== Phase 34b: Bytecode VM Performance Demo ===\n\n');

% --- Benchmark 1: scalar accumulator loop ---
fprintf('Benchmark 1: sum 1..10000 via for loop\n');
tic();
s = 0;
for k = 1:10000
  s += k;
end
t = toc();
fprintf('  s = %g  (expected %g)\n', s, 10000 * 10001 / 2);
fprintf('  time: %.3f ms\n\n', t * 1000);

% --- Benchmark 2: nested loop ---
fprintf('Benchmark 2: 100x100 nested loop\n');
tic();
total = 0;
for i = 1:100
  for j = 1:100
    total += 1;
  end
end
t = toc();
fprintf('  total = %g  (expected 10000)\n', total);
fprintf('  time: %.3f ms\n\n', t * 1000);

% --- Benchmark 3: while loop ---
fprintf('Benchmark 3: while loop x < 1000\n');
tic();
x = 0;
while x < 1000
  x += 1;
end
t = toc();
fprintf('  x = %g  (expected 1000)\n', x);
fprintf('  time: %.3f ms\n\n', t * 1000);

% --- Benchmark 4: function call loop ---
fprintf('Benchmark 4: 1000 calls to a trivial function\n');

function y = inc(x)
  y = x + 1;
end

tic();
s = 0;
for k = 1:1000
  s = inc(s);
end
t = toc();
fprintf('  s = %g  (expected 1000)\n', s);
fprintf('  time: %.3f ms\n\n', t * 1000);

fprintf('=== Done ===\n');
