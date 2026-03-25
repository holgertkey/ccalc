# ccalc

A command-line calculator with a persistent accumulator, memory cells, and math functions.

**Current version: 0.7.0** — see [CHANGELOG](CHANGELOG.md) for history.

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

### Pipe / non-interactive mode

When stdin is not a terminal, ccalc runs silently — no prompt, one result per line. The accumulator carries over across lines, so you can chain calculations:

```
$ echo "sin(pi / 6)" | ccalc
0.5

$ printf "10\n+ 5\n* 2" | ccalc
10
15
30

$ ccalc < formulas.txt
```

All commands work in pipe mode: `q` stops processing, `c` resets the accumulator, `mc`/`mc[1-9]` clear memory, `m[1-9]` stores into a cell, `p`/`p<N>` set precision, `hex`/`dec`/`bin`/`oct`/`base` control number base. `cls` and `m` are ignored.

---

## How it works

The prompt shows the **accumulator** — the result of the last expression. Every new expression updates it. Expressions that start with an operator automatically use the accumulator as the left-hand operand (**partial expressions**):

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

| Operator    | Description               | Precedence |
|-------------|---------------------------|------------|
| `^`         | Power (right-associative) | highest    |
| `*` `/` `%` | Multiply, divide, modulo  | medium     |
| `+` `-`     | Add, subtract             | lowest     |

```
[ 0 ]: 2 + 3 * 4
[ 14 ]:

[ 0 ]: 2 ^ 3 ^ 2
[ 512 ]:               (right-associative: 2^(3^2) = 2^9)

[ 0 ]: 17 % 5
[ 2 ]:
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

### Percentage operator

`N%` means *N percent of the accumulator* — a postfix operator that expands to `N * acc / 100`:

```
[ 1500 ]: 20%
[ 300 ]:               (20% of 1500)

[ 1500 ]: + 20%
[ 1800 ]:              (1500 + 20% of 1500)

[ 1800 ]: - 10%
[ 1620 ]:              (1800 − 10% of 1800)
```

`%` still works as **modulo** when followed by a number or expression:

```
[ 0 ]: 17 % 5
[ 2 ]:
```

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

| Name  | Value                     |
|-------|---------------------------|
| `pi`  | 3.14159265358979...       |
| `e`   | 2.71828182845904...       |
| `acc` | Current accumulator value |

`acc` is an explicit alias for the accumulator — useful when you need it in the middle of an expression:

```
[ 9 ]: acc * 2 + 1
[ 19 ]:

[ 25 ]: sqrt(acc)
[ 5 ]:
```

---

## Math functions

All functions take a single argument in parentheses. If called with **empty parentheses**, the accumulator is used as the argument.

| Function   | Description       |
|------------|-------------------|
| `sqrt(x)`  | Square root       |
| `abs(x)`   | Absolute value    |
| `floor(x)` | Round down        |
| `ceil(x)`  | Round up          |
| `round(x)` | Round to nearest  |
| `log(x)`   | Base-10 logarithm |
| `ln(x)`    | Natural logarithm |
| `exp(x)`   | *e* raised to *x* |
| `sin(x)`   | Sine (radians)    |
| `cos(x)`   | Cosine (radians)  |
| `tan(x)`   | Tangent (radians) |

```
[ 0 ]: sqrt(144)
[ 12 ]:

[ 0 ]: sin(pi / 6)
[ 0.5 ]:

[ 0 ]: log(1000)
[ 3 ]:

[ 16 ]: sqrt()          same as sqrt(16)
[ 4 ]:

[ 4 ]: sqrt(acc)        same as sqrt(4)
[ 2 ]:
```

Functions can be nested and combined:

```
[ 0 ]: sqrt(abs(-25))
[ 5 ]:

[ 0 ]: round(sin(pi / 3) * 100) / 100
[ 0.87 ]:
```

---

## Memory cells

Nine memory cells: `m1` through `m9`. Values persist for the duration of the session and can be saved to disk with `ms` and restored with `ml`.

### Store

| Input     | Action                                      |
|-----------|---------------------------------------------|
| `m1`      | Store accumulator into `m1`                 |
| `expr m1` | Evaluate expression, store result into `m1` |

```
[ 0 ]: 42
[ 42 ]: m1              m1 = 42

[ 0 ]: (10 + 5) * 2 m2  m2 = 30
[ 30 ]:
```

### Recall

Use `m1`–`m9` as values anywhere inside an expression:

```
[ 0 ]: m1 + m2
42 + 30
[ 72 ]:                 (42 + 30)

[ 0 ]: m1 * 2 + m2 / 3
42 * 2 + 30 / 3
[ 94 ]:
```

When memory references are expanded, the substituted expression is printed before the result:

```
[ 0 ]: m1 + 8 + m1
6 + 8 + 6
[ 20 ]:
```

### Compound assignment

`expr m[1-9]OP` evaluates `cell OP expr`, stores the result back into the cell, and sets the accumulator to the new cell value.

| Directive | Effect |
|-----------|--------|
| `expr m1+` | `m1 += expr` |
| `expr m1-` | `m1 -= expr` |
| `expr m1*` | `m1 *= expr` |
| `expr m1/` | `m1 /= expr` |
| `expr m1%` | `m1 %= expr` |
| `expr m1^` | `m1 ^= expr` |

```
[ 0 ]: 100 m1           m1 = 100; accumulator = 100
[ 100 ]: 2 m1*          m1 = 200; accumulator = 200
[ 200 ]: 50 m1-         m1 = 150; accumulator = 150
[ 150 ]: 3 m1/          m1 = 50;  accumulator = 50
```

The expression itself can be anything, including memory references:

```
[ 0 ]: m2 m1+           m1 = m1 + m2
[ 0 ]: 1 m1+            m1 += 1   (increment)
```

### Copy cell to cell

```
[ 0 ]: m1 m2            store value of m1 into m2
```

### View, clear, and persist

| Command | Action                                              |
|---------|-----------------------------------------------------|
| `m`     | Show all non-zero cells                             |
| `mc`    | Clear all cells                                     |
| `mc1`   | Clear cell `m1`                                     |
| `ms`    | Save all cells to `~/.config/ccalc/memory.toml`     |
| `ml`    | Load cells from file (clears current cells first)   |

```
[ 10 ]: m
m1: 85
m2: 30
[ 10 ]: mc1
[ 10 ]: mc
```

---

## REPL commands

| Command         | Action                                      |
|-----------------|---------------------------------------------|
| `q`             | Quit                                        |
| `c`             | Clear accumulator (reset to 0)              |
| `cls`           | Clear the screen                            |
| `p`             | Show current decimal precision              |
| `p<N>`          | Set decimal precision (0–15)                |
| `hex` / `dec` / `bin` / `oct` | Switch display base           |
| `base`          | Show accumulator in all four bases          |
| `ms`            | Save memory cells to disk                   |
| `ml`            | Load memory cells from disk                 |
| Ctrl+C / Ctrl+D | Quit                                        |

## Keyboard shortcuts

| Key                | Action                        |
|--------------------|-------------------------------|
| ↑ / ↓              | Browse input history          |
| Ctrl+R             | Reverse history search        |
| ← / → / Home / End | Cursor movement               |
| Ctrl+W             | Delete word before cursor     |
| Ctrl+U             | Clear line                    |

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
| `base`  | Show accumulator in all four bases  |

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

**Percentage — add VAT:**

```
[ 0 ]: 1200
[ 1200 ]: + 20%
[ 1440 ]:              (1200 + 20% of 1200)
```

**Percentage — discount:**

```
[ 0 ]: 850
[ 850 ]: - 15%
[ 722.5 ]:             (850 − 15% of 850)
```

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

**Running budget** — track a total across multiple entries:

```
[ 0 ]: 1200 m1          budget in m1
[ 1200 ]: m1 - 350 m1   spent 350 → m1 = 850
1200 - 350
[ 850 ]: 80 m1-         spent 80  → m1 = 770
850 - 80
[ 770 ]: m
m1: 770
```

**Angle conversion** — degrees to radians, then sine:

```
[ 0 ]: 30 * pi / 180
[ 0.5235987756 ]: sin()
[ 0.5 ]:
```

**Storing intermediate results**:

```
[ 0 ]: sqrt(2) m1       store √2
[ 1.4142135624 ]: m1 ^ 10
[ 32 ]:                 (√2)^10 = 2^5 = 32
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
src/
  main.rs      — entry point, mode detection (arg / pipe / REPL), CLI flags
  repl.rs      — REPL loop, run_pipe(), run_expr(), shared evaluate() core
  parser.rs    — lexer (tokenizer) + recursive descent parser
  eval.rs      — AST types (Expr, Op) + evaluator + number formatters + Base enum
  memory.rs    — memory cells, command parser, directive extractor, ref expander
Cargo.toml     — manifest (single source of truth for version)
CHANGELOG.md   — version history
```

---

## License

MIT
