% cell_arrays.calc — Phase 12.5 (cell arrays, varargin/varargout,
%                                  cellfun/arrayfun, @funcname handles)
%
% Topic: heterogeneous containers, variadic functions, and higher-order
%        operations on cells and numeric vectors.
%        All expected results are annotated for easy verification.
%
% Usage: ccalc cell_arrays.calc

% ── 1. Cell literal and brace-indexing ──────────────────────────────────────

fprintf('=== 1. Cell literal and indexing ===\n')

mixed = {42, 'hello', 3.14, [1 2 3]};

fprintf('  mixed{1} = %g       (exact: 42)\n',    mixed{1})
fprintf('  mixed{2} = %s       (exact: hello)\n',  mixed{2})
fprintf('  mixed{3} = %.2f     (exact: 3.14)\n',   mixed{3})
fprintf('  numel    = %d       (exact: 4)\n',       numel(mixed))
fprintf('  length   = %d       (exact: 4)\n',       length(mixed))
fprintf('  iscell   = %d       (exact: 1)\n',       iscell(mixed))
fprintf('  iscell(7)= %d       (exact: 0)\n',       iscell(7))

% ── 2. Building a cell with cell() and assignment ────────────────────────────

fprintf('\n=== 2. Growing a cell with assignment ===\n')

c = cell(3);          % 1x3 cell pre-filled with 0
c{1} = 'first';
c{2} = 99;
c{3} = [10 20 30];
c{4} = 'auto-grown';  % extends beyond initial size

fprintf('  c{1} = %s    (exact: first)\n',     c{1})
fprintf('  c{2} = %g       (exact: 99)\n',     c{2})
fprintf('  c{4} = %s  (exact: auto-grown)\n',  c{4})
fprintf('  numel= %d       (exact: 4)\n',       numel(c))

% ── 3. @funcname function handles ───────────────────────────────────────────

fprintf('\n=== 3. @funcname handles ===\n')

f_sqrt = @sqrt;
f_abs  = @abs;
f_sin  = @sin;

fprintf('  @sqrt(16)   = %g   (exact: 4)\n',      f_sqrt(16))
fprintf('  @abs(-7.5)  = %g   (exact: 7.5)\n',    f_abs(-7.5))
fprintf('  @sin(pi/2)  = %g   (exact: 1)\n',      f_sin(pi/2))

% Composing handles via a lambda
compose = @(f, g) @(x) f(g(x));
sqrt_abs = compose(@sqrt, @abs);
fprintf('  sqrt(abs(-9))= %g  (exact: 3)\n',      sqrt_abs(-9))

% ── 4. cellfun — apply a function to every cell element ──────────────────────

fprintf('\n=== 4. cellfun ===\n')

nums = {1, 4, 9, 16, 25};

% Using a @funcname handle — result is a numeric row vector when all scalar
roots = cellfun(@sqrt, nums);
fprintf('  cellfun(@sqrt, {1,4,9,16,25}):\n')
fprintf('    [')
for k = 1:length(roots)
  fprintf(' %g', roots(k))
end
fprintf(' ]   (exact: [ 1 2 3 4 5 ])\n')

% Using a lambda
doubled = cellfun(@(x) x * 2, nums);
fprintf('  cellfun(@(x) x*2, ...):\n')
fprintf('    [')
for k = 1:length(doubled)
  fprintf(' %g', doubled(k))
end
fprintf(' ]   (exact: [ 2 8 18 32 50 ])\n')

% ── 5. arrayfun — apply a function to every vector element ───────────────────

fprintf('\n=== 5. arrayfun ===\n')

v = [1 2 3 4 5];

squares = arrayfun(@(x) x^2, v);
fprintf('  arrayfun(@(x) x^2, 1:5):\n')
fprintf('    [')
for k = 1:length(squares)
  fprintf(' %g', squares(k))
end
fprintf(' ]   (exact: [ 1 4 9 16 25 ])\n')

% Combining arrayfun with a named function
function y = clamp01(x)
  if x < 0
    y = 0;
  elseif x > 1
    y = 1;
  else
    y = x;
  end
end

clamped = arrayfun(@clamp01, [-0.5 0 0.3 0.8 1.5]);
fprintf('  arrayfun(@clamp01, [-0.5 0 0.3 0.8 1.5]):\n')
fprintf('    [')
for k = 1:length(clamped)
  fprintf(' %g', clamped(k))
end
fprintf(' ]   (exact: [ 0 0 0.3 0.8 1 ])\n')

% ── 6. varargin — variadic input arguments ───────────────────────────────────

fprintf('\n=== 6. varargin ===\n')

function s = sum_all(varargin)
  s = 0;
  for k = 1:numel(varargin)
    s += varargin{k};
  end
end

fprintf('  sum_all(1)         = %g   (exact: 1)\n',  sum_all(1))
fprintf('  sum_all(1,2,3)     = %g   (exact: 6)\n',  sum_all(1, 2, 3))
fprintf('  sum_all(10,20,30)  = %g   (exact: 60)\n', sum_all(10, 20, 30))

% Mixed fixed + variadic
function display_tagged(label, varargin)
  fprintf('  [%s]', label)
  for k = 1:numel(varargin)
    fprintf(' %g', varargin{k})
  end
  fprintf('\n')
end

fprintf('  display_tagged output:\n')
display_tagged('A', 1, 2, 3)
display_tagged('B', 100)
% Expected:
%   [A] 1 2 3
%   [B] 100

% ── 7. varargout — variadic output arguments ─────────────────────────────────

fprintf('\n=== 7. varargout ===\n')

function varargout = first_n(v, n)
  for k = 1:n
    varargout{k} = v(k);
  end
end

[a, b, c] = first_n([10 20 30 40], 3);
fprintf('  first_n([10 20 30 40], 3) -> %g %g %g   (exact: 10 20 30)\n', a, b, c)

% ── 8. switch with cell case — multi-value matching ──────────────────────────

fprintf('\n=== 8. switch — case with cell array ===\n')

function describe(x)
  switch x
    case {1, 2, 3}
      fprintf('  %g -> small (1-3)\n', x)
    case {4, 5, 6}
      fprintf('  %g -> medium (4-6)\n', x)
    otherwise
      fprintf('  %g -> large (>6)\n', x)
  end
end

for val = [1 3 5 9]
  describe(val)
end
% Expected:
%   1 -> small (1-3)
%   3 -> small (1-3)
%   5 -> medium (4-6)
%   9 -> large (>6)

% ── 9. Practical example — pipeline of transformations ───────────────────────
%
% Build a pipeline from a cell array of function handles and apply it
% sequentially to a value.

fprintf('\n=== 9. Function pipeline ===\n')

function y = apply_pipeline(x, pipeline)
  y = x;
  for k = 1:numel(pipeline)
    f = pipeline{k};   % store handle in a variable so f(y) works
    y = f(y);
  end
end

pipeline = {@(x) x + 1, @(x) x * 2, @sqrt};

% (5 + 1) * 2 = 12  then  sqrt(12) ~ 3.4641
result = apply_pipeline(5, pipeline);
fprintf('  pipeline(5): (5+1)*2=12, sqrt(12) = %.4f   (exact: 3.4641)\n', result)

pipeline2 = {@abs, @(x) x^2, @(x) x - 1};
% abs(-3)=3, 3^2=9, 9-1=8
result2 = apply_pipeline(-3, pipeline2);
fprintf('  pipeline(-3): abs->sq->-1 = %g   (exact: 8)\n', result2)

fprintf('\nDone.\n')
