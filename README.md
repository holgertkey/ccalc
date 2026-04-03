# ccalc

A fast terminal calculator with Octave/MATLAB syntax and script support — one binary, no runtime.

**Current version: 0.10.0** — see [CHANGELOG](CHANGELOG.md) for history.

---

## Why ccalc?

Octave is hundreds of megabytes. Python requires a runtime. ccalc is a single
self-contained binary that starts instantly and works anywhere: interactive
sessions, shell scripts, CI pipelines, Docker containers.

It speaks Octave/MATLAB syntax — familiar to engineers and scientists — without
requiring a full language installation.

| Who                         | Typical use                                           |
|-----------------------------|-------------------------------------------------------|
| Embedded / systems engineer | Arithmetic, hex/bin conversions, bit masks            |
| DevOps / SRE                | Quick calculations in scripts and pipelines           |
| Scientist / student         | Interactive session with variables and math functions |
| MATLAB / Octave user        | Familiar syntax, no heavy installation                |

---

## Installation

```bash
git clone https://github.com/holgertkey/ccalc
cd ccalc
cargo build --release
```

The binary is placed at `target/release/ccalc`. Copy it anywhere on your `PATH`.

---

## Usage

```
ccalc [OPTIONS]           start interactive REPL
ccalc "EXPR"              evaluate expression and print result
ccalc script.m            run a script file
echo "EXPR" | ccalc       pipe mode — silent, result only
ccalc < formulas.txt      read expressions from file
```

| Option            | Description  |
|-------------------|--------------|
| `-h`, `--help`    | Show help    |
| `-v`, `--version` | Show version |

---

## Modes

### Interactive REPL

Run without arguments:

```
$ ccalc
[ 0 ]:
```

### Single expression

Pass an expression as a command-line argument:

```
$ ccalc "2 ^ 32"
4294967296

$ ccalc "sqrt(144)"
12
```

### Script file

Pass a script file as an argument — any file that exists on disk:

```
$ ccalc script.m
$ ccalc examples/mortgage.ccalc
```

### Pipe / non-interactive mode

When stdin is not a terminal, ccalc runs silently — no prompt, one result per line. `ans` carries over across lines, so you can chain calculations:

```
$ echo "sin(pi / 6)" | ccalc
0.5

$ printf "10\n+ 5\n* 2" | ccalc
10
15
30

$ ccalc < formulas.txt
```

All commands work in script/pipe mode: `exit`/`quit` stop processing, `who`/`clear`/`ws`/`wl` manage variables, `p`/`p<N>` set precision, `hex`/`dec`/`bin`/`oct`/`base` control number base. `cls` is ignored.

---

## How it works

The prompt shows **ans** — the result of the last expression. Every new expression updates it. Expressions that start with an operator automatically use `ans` as the left-hand operand (**partial expressions**):

```
[ 0 ]: 100
[ 100 ]: / 4
[ 25 ]: + 5
[ 30 ]: ^ 2
[ 900 ]:
```

---

## Arithmetic

### Operators

| Operator | Description               | Precedence |
|----------|---------------------------|------------|
| `^`      | Power (right-associative) | highest    |
| `*` `/`  | Multiply, divide          | medium     |
| `+` `-`  | Add, subtract             | lowest     |

```
[ 0 ]: 2 + 3 * 4
[ 14 ]:

[ 0 ]: 2 ^ 3 ^ 2
[ 512 ]:               (right-associative: 2^(3^2) = 2^9)
```

### Grouping

```
[ 0 ]: (2 + 3) * 4
[ 20 ]:
```

### Unary minus

```
[ 0 ]: -5
[ -5 ]:

[ 0 ]: -(3 + 2)
[ -5 ]:
```

---

## Ergonomics

### Implicit multiplication

A number or closing parenthesis immediately before `(` multiplies without an explicit `*`:

```
[ 0 ]: 2(3 + 1)
[ 8 ]:

[ 0 ]: (2 + 1)(4 - 1)
[ 9 ]:
```

---

## Constants

| Name  | Value                         |
|-------|-------------------------------|
| `pi`  | 3.14159265358979...           |
| `e`   | 2.71828182845904...           |
| `ans` | Result of the last expression |

`ans` is the implicit accumulator — it is updated after every expression and can be used anywhere in an expression:

```
[ 9 ]: ans * 2 + 1
[ 19 ]:

[ 25 ]: sqrt(ans)
[ 5 ]:
```

---

## Math functions

If called with **empty parentheses**, `ans` is used as the argument.

### One-argument

| Function   | Description                      |
|------------|----------------------------------|
| `sqrt(x)`  | Square root                      |
| `abs(x)`   | Absolute value                   |
| `floor(x)` | Round down to integer            |
| `ceil(x)`  | Round up to integer              |
| `round(x)` | Round to nearest integer         |
| `sign(x)`  | Sign: −1, 0, or 1                |
| `log(x)`   | Base-10 logarithm                |
| `ln(x)`    | Natural logarithm (base *e*)     |
| `exp(x)`   | *e* raised to the power *x*      |
| `sin(x)`   | Sine (radians)                   |
| `cos(x)`   | Cosine (radians)                 |
| `tan(x)`   | Tangent (radians)                |
| `asin(x)`  | Inverse sine (radians)           |
| `acos(x)`  | Inverse cosine (radians)         |
| `atan(x)`  | Inverse tangent (radians)        |

### Two-argument

| Function        | Description                                             |
|-----------------|---------------------------------------------------------|
| `atan2(y, x)`   | Four-quadrant inverse tangent (radians)                 |
| `mod(a, b)`     | Remainder, sign follows divisor (Octave convention)     |
| `rem(a, b)`     | Remainder, sign follows dividend                        |
| `max(a, b)`     | Larger of two values                                    |
| `min(a, b)`     | Smaller of two values                                   |
| `hypot(a, b)`   | √(a²+b²), numerically stable                           |
| `log(x, base)`  | Logarithm of *x* to an arbitrary *base*                 |

```
[ 0 ]: sqrt(144)
[ 12 ]:

[ 0 ]: sin(pi / 6)
[ 0.5 ]:

[ 0 ]: hypot(3, 4)
[ 5 ]:

[ 0 ]: atan2(1, 1) * 180 / pi
[ 45 ]:

[ 0 ]: mod(-1, 3)
[ 2 ]:

[ 16 ]: sqrt()          same as sqrt(16)
[ 4 ]:
```

Functions can be nested and combined:

```
[ 0 ]: sqrt(abs(-25))
[ 5 ]:

[ 0 ]: max(hypot(3, 4), 6)
[ 6 ]:
```

---

## Variables

Any identifier can be used as a variable. `ans` is the implicit variable
updated after every standalone expression.

### Assignment

`name = expr` shows the assigned value and does **not** update `ans`.
Append `;` to suppress output.

```
[ 0 ]: rate = 0.06 / 12
rate = 0.005
[ 0 ]: n = 360
n = 360
[ 0 ]: 200000 * 0.005
[ 1000 ]:
```

### Using variables

```
[ 0 ]: rate = 0.07
rate = 0.07
[ 0 ]: 1000 * (1 + rate) ^ 10
[ 1967.1513573 ]:
```

### View and clear

| Command       | Action                                             |
|---------------|----------------------------------------------------|
| `who`         | Show all defined variables and their values        |
| `clear`       | Clear all variables                                |
| `clear name`  | Clear a single variable                            |
| `ws`          | Save workspace to `~/.config/ccalc/workspace.toml` |
| `wl`          | Load workspace from file                           |

```
[ 0 ]: rate = 0.05
[ 0 ]: n = 12
[ 0 ]: rate + n
[ 12.05 ]: who
ans = 12.05
n = 12
rate = 0.05
[ 12.05 ]: clear rate
[ 12.05 ]: clear
```

---

## Matrices

Create matrices using bracket syntax. Separate elements with spaces or commas;
separate rows with `;`:

```
[ 0 ]: A = [1 2; 3 4]
A =
   1   2
   3   4

[ [2×2] ]: B = [5 6; 7 8]
B =
   5   6
   7   8

[ [2×2] ]: A + B
ans =
    6    8
   10   12
```

**Scalar operations** are element-wise:

```
[ [2×2] ]: 2 * A
ans =
   2   4
   6   8
```

### Matrix multiplication and transpose

```
[ [2×2] ]: A * B          matrix multiplication
[ [2×2] ]: A'             transpose
[ [2×2] ]: v' * v         dot product (row × column)
```

### Element-wise operators

```
[ [2×2] ]: A .* B         element-wise multiply
[ [2×2] ]: A ./ B         element-wise divide
[ [2×2] ]: A .^ 2         element-wise power
```

### Matrix built-ins

| Function        | Description                          |
|-----------------|--------------------------------------|
| `zeros(m, n)`   | All-zeros matrix                     |
| `ones(m, n)`    | All-ones matrix                      |
| `eye(n)`        | n×n identity matrix                  |
| `size(A)`       | `[rows cols]` as a 1×2 matrix        |
| `size(A, dim)`  | Number of rows (dim=1) or cols (dim=2)|
| `length(A)`     | `max(rows, cols)`                    |
| `numel(A)`      | Total number of elements             |
| `trace(A)`      | Sum of diagonal elements             |
| `det(A)`        | Determinant                          |
| `inv(A)`        | Inverse matrix                       |

The REPL prompt shows the matrix dimensions when `ans` is a matrix.
`who` displays dimensions: `A = [2×2 double]`.
`ws` saves only scalar variables; matrices are not persisted.

### Range operator

Generate row vectors with the `:` operator:

```
1:5              % [1 2 3 4 5]
1:2:9            % [1 3 5 7 9]   (start:step:stop)
0:0.5:2          % [0 0.5 1 1.5 2]
5:-1:1           % [5 4 3 2 1]
```

Ranges work inside matrix literals and can be mixed with scalars:

```
[ 0 ]: v = 1:4
v =
   1   2   3   4

[ [1×4] ]: [0, 1:3, 10]
ans =
    0   1   2   3   10
```

`linspace(a, b, n)` generates `n` evenly spaced values between `a` and `b`:

```
linspace(0, 1, 5)    % [0  0.25  0.5  0.75  1]
linspace(1, 5, 5)    % [1  2  3  4  5]
```

---

## REPL commands

| Command                           | Action                              |
|-----------------------------------|-------------------------------------|
| `exit`, `quit`                    | Quit                                |
| `cls`                             | Clear the screen                    |
| `help`, `?`                       | Show cheatsheet                     |
| `help <topic>`                    | Detailed help (see below)           |
| `who`                             | List all defined variables          |
| `clear`                           | Clear all variables                 |
| `clear <name>`                    | Clear a single variable             |
| `p`                               | Show current decimal precision      |
| `p<N>`                            | Set decimal precision (0–15)        |
| `hex` / `dec` / `bin` / `oct`     | Switch display base                 |
| `base`                            | Show ans in all four bases          |
| `ws`                              | Save workspace to disk              |
| `wl`                              | Load workspace from disk            |
| Ctrl+C / Ctrl+D                   | Quit                                |

Help topics: `syntax`  `functions`  `bases`  `vars`  `script`  `matrices`  `examples`

## Keyboard shortcuts

| Key                | Action                                    |
|--------------------|-------------------------------------------|
| ↑ / ↓              | Browse input history                      |
| Ctrl+R             | Reverse history search                    |
| ← / → / Home / End | Cursor movement                           |
| Ctrl+A             | Go to beginning of line                   |
| Ctrl+E             | Go to end of line                         |
| Ctrl+W             | Delete word before cursor                 |
| Ctrl+U             | Delete from cursor to beginning of line   |
| Ctrl+K             | Delete from cursor to end of line         |
| Ctrl+L             | Clear screen                              |
| Ctrl+C / Ctrl+D    | Quit                                      |

---

## Number formatting and bases

### Decimal precision

By default results are shown with up to 10 decimal digits, trailing zeros removed:

```
[ 0 ]: 1 / 3
[ 0.3333333333 ]:
[ 0 ]: p4
[ 0 ]: 1 / 3
[ 0.3333 ]:
[ 0 ]: p          show current precision
precision: 4
```

`p<N>` sets N decimal places (0–15). `p` alone shows the current setting.

Very large (`|n| >= 1e15`) and very small (`|n| < 1e-9`) numbers switch to scientific notation automatically:

```
[ 0 ]: 2 ^ 60
[ 1.152921504606847e18 ]:
```

### Number bases

**Input literals** — mix bases freely in any expression:

| Prefix | Base   | Example        |
|--------|--------|----------------|
| `0x`   | hex    | `0xFF` → 255   |
| `0b`   | binary | `0b1010` → 10  |
| `0o`   | octal  | `0o17` → 15    |

**Display base** — controls how the prompt and results are shown:

| Command | Effect                              |
|---------|-------------------------------------|
| `hex`   | Switch display to hexadecimal       |
| `dec`   | Switch display to decimal (default) |
| `bin`   | Switch display to binary            |
| `oct`   | Switch display to octal             |
| `base`  | Show ans in all four bases          |

```
[ 0 ]: 0xFF + 0b1010
[ 265 ]: hex
[ 0x109 ]: + 0b10
[ 0x10B ]: dec
[ 267 ]:
```

**Inline base suffix** — evaluate an expression and switch display base in one step:

```
[ 0 ]: 0xFF + 0b1010 hex
[ 0x109 ]:
```

**`base` command:**

```
[ 10 ]: base
2  - 0b1010
8  - 0o12
10 - 10
16 - 0xA
```

**Expression conversion** — when the display base is non-decimal and the expression contains literals in other bases, the converted expression is printed before the result:

```
[ 0x6 ]: 0b11 + 0b11
0x3 + 0x3
[ 0x6 ]:

[ 0b110 ]: 2 + 0b110 + 0xa
0b10 + 0b110 + 0b1010
[ 0b10010 ]:
```

---

## Examples

**Implicit multiplication:**

```
[ 0 ]: 2(3 + 1)
[ 8 ]:

[ 0 ]: (2 + 1)(4 - 1)
[ 9 ]:
```

**Compound interest** — 1000 at 7% for 10 years:

```
[ 0 ]: 1000 * 1.07 ^ 10
[ 1967.15135729 ]:
```

**Pythagorean hypotenuse** — sides 3 and 4:

```
[ 0 ]: sqrt(3^2 + 4^2)
[ 5 ]:
```

**Variables — monthly mortgage:**

```
[ 0 ]: rate = 0.06 / 12
rate = 0.005
[ 0 ]: n = 360
n = 360
[ 0 ]: factor = (1 + rate) ^ n
factor = 10.9357...
[ 0 ]: 200000 * rate * factor / (factor - 1)
[ 1199.1010503 ]:
```

**Angle conversion** — degrees to radians, then sine:

```
[ 0 ]: 30 * pi / 180
[ 0.5235987756 ]: sin()
[ 0.5 ]:
```

---

## Script files

When reading from a file (`ccalc < formula.txt`) you have three tools to control output:

### Comments

`%` starts a comment (Octave/MATLAB convention). It can be the first character on the line (full-line comment) or appear after an expression (inline comment). Everything from `%` to end-of-line is ignored.

```
% Cylinder volume: V = pi * r^2 * h
pi * 5^2      % pi * r^2, r = 5
```

### Semicolon — suppress output

A trailing `;` evaluates the expression and updates `ans`, but prints nothing.
Use it to silence intermediate steps.

```
rate = 0.06 / 12;     % monthly rate — silent
n = 360;              % 30-year term — silent
factor = (1 + rate) ^ n;
200000 * rate * factor / (factor - 1)
fprintf('Monthly payment ($): ')
disp(ans)
```

### `disp(expr)` — print value

`disp(expr)` evaluates the expression and prints the result.
It does **not** update `ans`.

```
disp(ans)             % print current ans
disp(rate * 12)       % print expression result
```

### `fprintf('fmt')` — print formatted text

`fprintf('fmt')` prints a string with escape sequences.
No newline is added automatically — include `\n` explicitly.

```
fprintf('=== Resistors in series ===\n')

100 + 220 + 470
fprintf('Total resistance (Ohm): ')
disp(ans)

fprintf('=== Parallel combination ===\n')

1/100 + 1/220;
^ -1
fprintf('Parallel resistance (Ohm): ')
disp(ans)
```

Output:

```
=== Resistors in series ===
790
Total resistance (Ohm): 790
=== Parallel combination ===
68.7500002148
Parallel resistance (Ohm): 68.7500002148
```

### Example files

The `examples/` directory contains annotated formula files ready to run:

| File                  | Description                                          |
|-----------------------|------------------------------------------------------|
| `cylinder.calc`       | Volume and surface area of a cylinder                |
| `mortgage.calc`       | Monthly mortgage payment                             |
| `data_storage.calc`   | Real GiB capacity of a "500 GB" drive                |
| `resistors.calc`      | Series, parallel resistance, voltage divider, power  |
| `ac_impedance.calc`   | AC impedance, phase angle, dB level, bit width       |
| `matrix_ops.calc`     | Rotation, linear system solve, element-wise ops      |
| `sequences.calc`      | Ranges, linspace, indexing, slicing, finite differences |

```bash
ccalc < examples/mortgage.ccalc
```

---

## Building and testing

```bash
cargo build            # debug build
cargo build --release  # optimized build
cargo test             # run all tests
```

---

## Project structure

```
crates/
  ccalc/src/
    main.rs      — entry point, mode detection (arg / pipe / REPL), CLI flags
    repl.rs      — REPL loop, run_pipe(), run_expr(), shared evaluate() core
    help.rs      — help text
  ccalc-engine/src/
    lib.rs       — crate root, public module exports
    env.rs       — Value enum (Scalar/Matrix), Env type (HashMap<String, Value>), workspace save/load
    eval.rs      — AST types (Expr, Op) + evaluator returning Value + number formatters + Base enum
    parser.rs    — lexer (tokenizer) + recursive descent parser, Stmt enum
Cargo.toml       — workspace manifest (single source of truth for version)
CHANGELOG.md     — version history
```

---

## License

MIT
