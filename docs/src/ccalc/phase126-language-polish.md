# Phase 12.6 — Language Polish and Small Completions

**Version:** v0.18.0+001  
**Prerequisite:** Phase 12.5 (cell arrays — required for `strsplit` return type)

This phase closes accumulated gaps that are each small in isolation but collectively
leave visible holes in Octave/MATLAB compatibility.

---

## 12.6a — Single-line blocks

`if`, `for`, `while`, and `switch` can now appear on a single line with their
body and closing `end` separated by semicolons:

```matlab
if x > 5; label = 'big'; end
for k = 1:5; total += k; end
while mod(n,2) == 0; n = n/2; end
switch day; case 1; name='Mon'; case 2; name='Tue'; otherwise; name='?'; end
```

**Implementation:** `is_single_line_block(line)` detects a complete block by
splitting on `;` and checking whether the last segment's leading keyword is
`end` or `until`. The REPL and pipe mode bypass block buffering for these lines.

---

## 12.6b — Line continuation `...`

A line ending with `...` (after stripping comments) continues on the next line:

```matlab
result = 1 + ...
         2 + ...
         3;               % result = 6

A = [1 2 3; ...
     4 5 6];              % 2×3 matrix

if value > 0 && ...
   value < 100
  disp('in range')
end
```

Three implementation points:
- **REPL**: `cont_buf` accumulates partial lines; not dispatched until continuation ends.
- **Pipe/file mode**: same `cont_buf` logic in `run_pipe`.
- **Block parser**: `join_line_continuations()` pre-pass in `parse_stmts` joins
  `...`-continued lines before statement splitting.

---

## 12.6c — Element-wise logical operators `&` and `|`

`&&` and `||` are short-circuit scalar operators. `&` and `|` are element-wise
operators that work on matrices and always evaluate both sides:

```matlab
a = [1 0 1 0];
b = [1 1 0 0];

a & b          % [1 0 0 0]   element-wise AND
a | b          % [1 1 1 0]   element-wise OR

% Logical mask — common pattern
v = [3, -1, 8, 0, 5, -2, 7];
mask = v > 0 & v < 6    % [1 0 0 0 1 0 0]
```

New tokens `Token::Amp` / `Token::Pipe` and `Op::ElemAnd` / `Op::ElemOr`.
New parse levels `parse_elem_or` and `parse_elem_and` sit between
`parse_logical_and` and `parse_comparison` in the precedence hierarchy.

---

## 12.6d — `xor` and `not` built-ins

```matlab
xor(1, 0)                       % 1
xor(0, 0)                       % 0
xor([1 0 1 0], [1 1 0 0])       % [0 1 1 0]

not(0)                           % 1   (alias for ~)
not(5)                           % 0
not([1 0 1])                     % [0 1 0]
```

---

## 12.6e — Lambda source display

Lambdas now display their source expression instead of `@<lambda>`:

```
>> f = @(x) x^2 + 1
f = @(x) x ^ 2 + 1

>> g = @(a, b) sqrt(a^2 + b^2)
g = @(a, b) sqrt(a ^ 2 + b ^ 2)

>> h = @sin
h = @sin
```

`LambdaFn` carries a second field with the source string, populated at parse
time by `expr_to_string()` which reconstructs a readable expression from the AST.

---

## 12.6f — String utilities

```matlab
% strsplit — returns a cell array
parts = strsplit('alpha,beta,gamma', ',')
parts{1}                          % 'alpha'
parts{2}                          % 'beta'

words = strsplit('hello world')   % split on whitespace
numel(words)                      % 2

% int2str — round to integer, return string
int2str(3.2)                      % '3'
int2str(3.7)                      % '4'
int2str(-1.5)                     % '-2'

% mat2str — matrix to MATLAB literal string
mat2str([1 2; 3 4])               % '[1 2;3 4]'
mat2str([10 20 30])               % '[10 20 30]'
```

`strsplit` returns `Value::Cell` of `Value::Str` — requires Phase 12.5.

---

## 12.6g — `.'` non-conjugate transpose

`A'` is the conjugate transpose (Hermitian) — it flips the sign of complex imaginary
parts. `A.'` is the plain transpose with no conjugation:

```matlab
% Real matrices — identical result
B = [1 2 3; 4 5 6];
B.'       % [1 4; 2 5; 3 6]   (same as B')

% Complex — different result
z = 3 + 4i
z'        % 3 - 4i   (conjugate)
z.'       % 3 + 4i   (plain)
```

`Token::DotApostrophe` is emitted when `.` is immediately followed by `'`.
`Expr::PlainTranspose(Box<Expr>)` evaluates identically to `Expr::Transpose` for
real values, but skips complex conjugation for `Value::Complex`.

---

## 12.6j — Minor syntax completions

### Unary `+`

`+x` is a no-op (returns `x` unchanged). Previously caused a parse error.

```matlab
+5             % 5
+(-3)          % -3
+[1 2 3]       % [1 2 3]
```

### `**` exponentiation alias

Octave accepts `**` as a synonym for `^`:

```matlab
2 ** 10        % 1024
3 ** 3         % 27
2 ** 0.5       % 1.41421...
```

### `,` as non-silent statement separator

A comma between statements is like a newline — the result is shown (unlike `;`
which suppresses output):

```matlab
a = 1, b = 2    % shows: a = 1  then  b = 2
a = 1; b = 2    % a silent, b shown
```

---

## Bug fixes (v0.18.0+001)

- **`4i` imaginary literal** — `3 + 4i` now works in pipe and file mode.
  The tokenizer's `push_imag_suffix()` helper emits `* i` tokens after any
  decimal literal followed immediately by `i` or `j`.

- **`B.';` split incorrectly** — `split_stmts` now recognises `.` as a
  transpose indicator, preventing `B.'` from being mis-parsed as a string start.

- **`...` in pipe mode** — `run_pipe` now has the same `cont_buf` logic as
  `run_repl`, so multi-line scripts using `...` work correctly.

---

## Example

```
ccalc examples/language_polish.calc
```

The example covers all ten sections with expected output annotations.
