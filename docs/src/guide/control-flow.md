# Control Flow

ccalc supports multi-line control flow blocks in both the interactive REPL and
in script/pipe mode. All block constructs use `end` as the closing keyword.

## REPL multi-line input

The REPL detects unclosed blocks and buffers incoming lines, displaying a
continuation prompt `  >>` until the block is complete. Press `Ctrl+C` to
cancel an incomplete block.

```
[ 0 ]:   for k = 1:3
  >>   fprintf('%d\n', k)
  >> end
1
2
3
```

## if / elseif / else

```matlab
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
fprintf('score %d -> grade %s\n', score, grade)
```

A condition is **truthy** when:

| Value type   | Truthy when                              |
|--------------|------------------------------------------|
| Scalar       | non-zero and not NaN                     |
| Matrix       | all elements non-zero and not NaN        |
| Str/StringObj | non-empty                               |
| Void         | never                                    |

## for

```matlab
for var = range_expr
  % body
end
```

The range expression is evaluated once. Iteration is column-by-column:

- **Row vector** → each element as a scalar
- **M×N matrix** → each column as an M×1 column vector

```matlab
% Simple range
for k = 1:5
  fprintf('%d\n', k)
end

% Step range
for k = 10:-2:0
  fprintf('%d ', k)   % 10 8 6 4 2 0
end
```

## while

```matlab
while cond
  % body
end
```

```matlab
x = 1.0;
while abs(x ^ 2 - 2) > 1e-12
  x = (x + 2 / x) / 2;
end
fprintf('sqrt(2) ≈ %.15f\n', x)
```

## break and continue

`break` exits the innermost loop immediately. `continue` skips to the next
iteration.

```matlab
for n = 1:20
  if mod(n, 2) == 0
    continue        % skip even numbers
  end
  if n > 9
    break           % stop after first odd > 9
  end
  fprintf('%d ', n)   % 1 3 5 7 9
end
```

## Compound assignment operators

| Operator | Equivalent to   |
|----------|-----------------|
| `x += e` | `x = x + e`    |
| `x -= e` | `x = x - e`    |
| `x *= e` | `x = x * e`    |
| `x /= e` | `x = x / e`    |
| `x++`    | `x = x + 1`    |
| `x--`    | `x = x - 1`    |
| `++x`    | `x = x + 1`    |
| `--x`    | `x = x - 1`    |

All forms desugar at parse time to a plain `Stmt::Assign` — no new AST nodes.
The RHS is a full expression: `x *= 2 + 3` desugars to `x = x * (2 + 3)`.

> **Limitation**: `++`/`--` are statement-level only. Using them inside a
> larger expression (`b = a - b--`) is not supported.

## switch / case / otherwise

```matlab
switch expr
  case val1
    % ...
  case val2
    % ...
  otherwise     % optional
    % ...
end
```

No fall-through — only the first matching case executes. Works with scalars
(exact `==`) and strings (`Str`/`StringObj` interchangeable). `break`/`continue`
propagate to the nearest enclosing loop.

```matlab
switch code
  case 200
    msg = 'OK';
  case 404
    msg = 'Not Found';
  otherwise
    msg = 'Unknown';
end
fprintf('%d: %s\n', code, msg)
```

## do...until

Octave post-test loop — body always runs at least once:

```matlab
do
  body
until (cond)
```

Parentheses around `cond` are optional. Closed by `until`, not `end`.
`break` and `continue` work as in `while`.

```matlab
x = 1;
do
  x *= 2;
until (x > 100)
fprintf('%d\n', x)   % 128
```

## run() / source()

Execute a script file in the current workspace. Variables defined in the script
persist in the caller's scope (MATLAB `run` semantics — not a function call):

```matlab
a = 252; b = 105;
run('euclid_helper')        % looks for euclid_helper.calc, then .m
fprintf('gcd = %d\n', g)    % g was set by the helper

source('euclid_helper')     % Octave alias — identical behaviour
```

Extension resolution for bare names: `.calc` is tried first (native ccalc
format), then `.m` (Octave/MATLAB compatibility).

## Examples

See the example scripts for self-contained demos:

```bash
ccalc examples/control_flow.calc           # if/for/while/break/continue
ccalc examples/extended_control_flow.calc  # switch/do-until/run/source
```
