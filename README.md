# ccalc

A command-line calculator with a persistent accumulator, memory cells, and math functions.

**Current version: 0.4.0** — see [CHANGELOG](CHANGELOG.md) for history.

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

All commands work in pipe mode: `q` stops processing, `c` resets the accumulator, `mc`/`mc[1-9]` clear memory, `m[1-9]` stores into a cell. `cls` and `m` are ignored.

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

Nine persistent memory cells: `m1` through `m9`. They hold values across expressions for the duration of the session.

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
[ 72 ]:                 (42 + 30)

[ 0 ]: m1 * 2 + m2 / 3
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

### View and clear

| Command | Action                  |
|---------|-------------------------|
| `m`     | Show all non-zero cells |
| `mc`    | Clear all cells         |
| `mc1`   | Clear cell `m1`         |

```
[ 10 ]: m
m1: 85
m2: 30
[ 10 ]: mc1
[ 10 ]: mc
```

---

## REPL commands

| Command       | Action                         |
|---------------|--------------------------------|
| `q`           | Quit                           |
| `c`           | Clear accumulator (reset to 0) |
| `cls`         | Clear the screen               |
| Ctrl+C / Ctrl+D | Quit                         |

## Keyboard shortcuts

| Key                | Action                        |
|--------------------|-------------------------------|
| ↑ / ↓              | Browse input history          |
| Ctrl+R             | Reverse history search        |
| ← / → / Home / End | Cursor movement               |
| Ctrl+W             | Delete word before cursor     |
| Ctrl+U             | Clear line                    |

---

## Number formatting

Results are displayed without unnecessary decoration:

- Integers are shown without a decimal point: `12`, `-5`, `1024`
- Floats are trimmed to 10 significant fractional digits with trailing zeros removed: `3.14`, `0.5`, `1.4142135624`
- `0.1 + 0.2` displays as `0.3`

---

## Examples

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
[ 850 ]: 80 m1-         spent 80  → m1 = 770
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
[ 1.4142135624 ]: acc ^ 10
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
  eval.rs      — AST types (Expr, Op) + evaluator + number formatter
  memory.rs    — memory cells, command parser, directive extractor, ref expander
Cargo.toml     — manifest (single source of truth for version)
CHANGELOG.md   — version history
```

---

## License

MIT
