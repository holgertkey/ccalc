% indexed_assignment.calc — Phase 15 (indexed assignment)
%
% Topic: modifying matrix elements in place — the feature that makes
%        loops over arrays and numerical algorithms possible.
%
% Covers:
%   15a — scalar and slice assignment, 2-D element/row/column/submatrix
%   15b — growing vectors (end+1, padding with zeros)
%   15c — cell element assignment (growing)
%   15d — logical (boolean mask) indexing for read and write
%
% Usage: ccalc indexed_assignment.calc

% ── 1. Basic element and slice assignment ────────────────────────────────────

fprintf('=== 1. Element and slice assignment ===\n')

v = zeros(1, 6)
v(3) = 42;
fprintf('v after v(3) = 42:          ')
disp(v)

v(1:2) = [10, 20];
fprintf('v after v(1:2) = [10 20]:   ')
disp(v)

v(4:6) = 99;          % scalar broadcast to all three positions
fprintf('v after v(4:6) = 99:        ')
disp(v)

v(:) = 0;             % reset all elements at once
fprintf('v after v(:) = 0:           ')
disp(v)

% ── 2. 2-D matrix assignment ─────────────────────────────────────────────────

fprintf('\n=== 2. 2-D matrix assignment ===\n')

A = zeros(4)

A(2, 3) = 7;
fprintf('A after A(2,3) = 7:\n')
disp(A)

A(:, 1) = [1; 2; 3; 4];
fprintf('A after A(:,1) = [1;2;3;4]:\n')
disp(A)

A(1, :) = [10, 20, 30, 40];
fprintf('A after A(1,:) = [10 20 30 40]:\n')
disp(A)

A(2:3, 2:3) = eye(2);
fprintf('A after A(2:3,2:3) = eye(2):\n')
disp(A)

% ── 3. Growing vectors ───────────────────────────────────────────────────────

fprintf('\n=== 3. Growing vectors ===\n')

% Build a vector element-by-element with end+1
squares = [];
for k = 1:8
  squares(end+1) = k^2;
end
fprintf('Squares 1..8 built with end+1:\n  ')
disp(squares)

% Out-of-range index pads with zeros
v = [1, 2, 3];
v(7) = 99;
fprintf('v = [1,2,3] after v(7) = 99: ')
disp(v)

% Auto-create from nothing
fib = [];
fib(1) = 1;
fib(2) = 1;
for k = 3:10
  fib(end+1) = fib(end) + fib(end-1);
end
fprintf('First 10 Fibonacci numbers:\n  ')
disp(fib)

% ── 4. Cell element assignment (growing) ────────────────────────────────────

fprintf('\n=== 4. Cell element assignment ===\n')

% c{end+1} grows the cell array — end resolves to current length
labels = {};
data   = {};
labels{end+1} = 'alpha';   data{end+1} = 1;
labels{end+1} = 'beta';    data{end+1} = 2;
labels{end+1} = 'gamma';   data{end+1} = 3;
labels{end+1} = 'delta';   data{end+1} = 4;
fprintf('Cell array built with end+1:\n')
for k = 1:4
  fprintf('  labels{%d} = %-6s  data{%d} = %g\n', k, labels{k}, k, data{k})
end

% ── 5. Logical (boolean mask) indexing ──────────────────────────────────────

fprintf('\n=== 5. Logical indexing ===\n')

% Read: select elements matching a condition
temps = [18, 22, 35, 12, 29, 41, 8, 33];
hot = temps(temps >= 30);
fprintf('Temperatures:   ')
disp(temps)
fprintf('Hot days (>=30): ')
disp(hot)

% Write: set elements matching a condition
temps(temps >= 30) = 30;       % cap all hot days at 30
fprintf('After capping at 30: ')
disp(temps)

% Mask from a separate variable
signal = [0.5, -1.2, 0.8, -0.3, 1.5, -2.0, 0.1];
noise  = signal < 0;           % logical mask for negative samples
signal(noise) = 0;             % zero out negative samples (half-wave rectifier)
fprintf('Half-wave rectified signal: ')
disp(signal)

% 2-D logical mask
fprintf('\n')
M = [1, 2, 3; 4, 5, 6; 7, 8, 9];
fprintf('M:\n')
disp(M)

fprintf('M(M > 5) — elements above 5 in column-major order: ')
disp(M(M > 5))

M(M > 5) = 0;
fprintf('M after zeroing elements > 5:\n')
disp(M)

% ── 6. Practical example: building and filtering vectors ─────────────────────

fprintf('\n=== 6. Building and filtering vectors ===\n')

% Collect even numbers 1..20 using end+1 inside an if block
evens = [];
for k = 1:20
  if mod(k, 2) == 0
    evens(end+1) = k;
  end
end
fprintf('Even numbers 1..20:  ')
disp(evens)
fprintf('Sum = %g,  mean = %g\n', sum(evens), mean(evens))

% Cap values above 12 at 12 using a logical mask
evens(evens > 12) = 12;
fprintf('After capping at 12: ')
disp(evens)
