% Phase 25 — Dynamic evaluation and timing

% ── eval: execute a string as code ─────────────────────────────────────────
eval('x = sqrt(2)')           % x is now defined in the workspace
disp(x)                       % 1.4142...

eval('disp(pi)')              % prints 3.14159...

% Build code string dynamically
code = sprintf('y = %g * 2', 3.14);
eval(code)
disp(y)                       % 6.28

% ── eval: dynamic variable naming ──────────────────────────────────────────
for k = 1:4
  eval(sprintf('v%d = k^2', k))
end
disp(v1)   % 1
disp(v2)   % 4
disp(v3)   % 9
disp(v4)   % 16

% ── eval: two-argument form (try / catch) ──────────────────────────────────
% Second argument runs if the first raises an error
eval('result = 1 / 0', 'result = Inf; disp(''division guarded'')')
disp(result)   % Inf

eval('no_such_var + 1', 'disp(''caught undefined variable'')')

% ── tic / toc: elapsed time ────────────────────────────────────────────────
tic
s = 0;
for k = 1:10000
  s = s + k;
end
t1 = toc;
fprintf('loop sum to 10000: elapsed = %.6f s\n', t1)

% Multiple toc calls after a single tic are valid
tic
a = rand(50, 50) * rand(50, 50);
t_after_mul = toc;
b = inv(a);
t_after_inv = toc;
fprintf('50x50 multiply: %.6f s,  +inv: %.6f s\n', t_after_mul, t_after_inv)

% ── eval inside a loop with tic/toc ────────────────────────────────────────
ops = {'sin(pi/4)', 'exp(1)', 'sqrt(2) * sqrt(2)'};
tic
for k = 1:3
  eval(sprintf('r = %s', ops{k}));
  fprintf('ops{%d} = %g\n', k, r)
end
fprintf('three eval calls: %.6f s\n', toc)
