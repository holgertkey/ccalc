# Cell Arrays

A cell array is a **heterogeneous 1-D container**: each element can hold any
value — scalar, matrix, string, complex number, function handle, or even
another cell array.

---

## Creating cell arrays

```matlab
c = {1, 'hello', [1 2 3]};    % cell literal — comma-separated expressions
d = cell(5);                   % 1×5 cell pre-filled with zeros
e = cell(2, 4);                % 1×8 cell pre-filled with zeros (1-D, m*n slots)
```

## Brace indexing — reading elements

Use `c{i}` (curly braces, 1-based) to retrieve the content of element `i`:

```matlab
c = {42, 'hello', [1 2 3]};

c{1}    % → 42       (scalar)
c{2}    % → hello    (char array)
c{3}    % → [1 2 3]  (matrix)
```

> **Note:** `c(i)` with round parentheses returns an error — brace indexing
> `c{i}` is required to get the element's value.

## Assigning to elements

```matlab
c{2} = 'world';      % replace existing element
c{5} = pi;           % auto-grows: elements 4–5 are zero-filled
numel(c)             % 5
```

## Predicates and size

| Function     | Description                                |
|--------------|--------------------------------------------|
| `iscell(c)`  | `1` if `c` is a cell array, else `0`       |
| `numel(c)`   | Number of elements                         |
| `length(c)`  | Same as `numel(c)` for 1-D cells           |
| `size(c)`    | `[1  numel(c)]` as a 1×2 matrix            |

---

## `varargin` — variadic input

Declare the last parameter as `varargin` to collect all extra call arguments
into a cell array:

```matlab
function s = sum_all(varargin)
  s = 0;
  for k = 1:numel(varargin)
    s += varargin{k};
  end
end

sum_all(1, 2, 3)        % 6
sum_all(10, 20)         % 30
sum_all()               % 0  (empty varargin cell)
```

Fixed and variadic parameters can be mixed:

```matlab
function show(label, varargin)
  fprintf('[%s]', label)
  for k = 1:numel(varargin)
    fprintf(' %g', varargin{k})
  end
  fprintf('\n')
end

show('A', 1, 2, 3)    % [A] 1 2 3
show('B', 100)         % [B] 100
```

---

## `varargout` — variadic output

Declare the sole output variable as `varargout` (a cell array) and the caller
receives one output value per cell element:

```matlab
function varargout = first_n(v, n)
  for k = 1:n
    varargout{k} = v(k);
  end
end

[a, b, c] = first_n([10 20 30 40], 3)   % a=10  b=20  c=30
```

---

## `case {v1, v2}` — multi-value switch cases

Inside a `switch` block, a cell array case matches if the switch expression
equals **any** element of the cell:

```matlab
switch x
  case {1, 2, 3}
    disp('small')
  case {4, 5, 6}
    disp('medium')
  otherwise
    disp('large')
end
```

---

## `cellfun` — apply a function to a cell

`cellfun(f, c)` applies `f` to each element of cell `c`.
Returns a `Matrix` when all results are scalar; otherwise returns a `Cell`.

```matlab
c = {1, 4, 9, 16, 25};
cellfun(@sqrt, c)             % [1  2  3  4  5]
cellfun(@(x) x * 2, c)       % [2  8  18  32  50]
```

---

## `arrayfun` — apply a function to a numeric vector

`arrayfun(f, v)` applies `f` to each element of matrix `v`.
Returns a same-shape matrix (function must return a scalar per element).

```matlab
arrayfun(@(x) x^2, [1 2 3 4])        % [1  4  9  16]
arrayfun(@(x) x > 2, [1 2 3 4])      % [0  0  1   1]
```

---

## `@funcname` — function handles

`@funcname` creates a callable that forwards its arguments to `funcname`.
Works with any built-in or user-defined function:

```matlab
f = @sqrt;
g = @abs;

f(16)     % 4
g(-7.5)   % 7.5

cellfun(@sqrt, {1, 4, 9})   % [1  2  3]
arrayfun(@abs, [-1 -2 3])   % [1  2  3]
```

Compose handles via a capturing lambda:

```matlab
compose = @(f, g) @(x) f(g(x));
sqrt_abs = compose(@sqrt, @abs);
sqrt_abs(-9)    % 3   ( sqrt(abs(-9)) )
```

---

## Function pipelines

Store a sequence of function handles in a cell array and apply them in order:

```matlab
function y = apply_pipeline(x, pipeline)
  y = x;
  for k = 1:numel(pipeline)
    f = pipeline{k};
    y = f(y);
  end
end

pipeline = {@(x) x + 1, @(x) x * 2, @sqrt};
apply_pipeline(5, pipeline)   % sqrt((5+1)*2) = sqrt(12) ≈ 3.4641
```

---

## Workspace

Cell arrays are **not** saved by `ws`/`save` — same policy as matrices and
complex values. `who` shows them as:

```
c = {1×4 cell}
```

---

## See also

- `help cells` — in-REPL reference
- `help userfuncs` — varargin/varargout in the context of user functions
- `ccalc examples/cell_arrays.calc` — annotated 9-section example
