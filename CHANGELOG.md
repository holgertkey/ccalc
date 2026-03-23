# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.3.0] - 2026-03-23

### Added

- Line editing via `rustyline`: ← → Home End cursor movement, Ctrl+W word delete, Ctrl+U line clear
- History navigation: ↑ ↓ to browse previous inputs, Ctrl+R for reverse search
- Ctrl+C and Ctrl+D as additional quit shortcuts (in addition to `q`)
- `acc` — explicit alias for the current accumulator value in expressions (e.g. `sqrt(acc)`, `acc * 2`)
- Empty function call `fn()` uses the accumulator as argument (e.g. `sqrt()` → `sqrt(accumulator)`)
- Compound assignment directives `m[1-9]OP` for operators `+`, `-`, `*`, `/`, `%`, `^`: `expr m1+` means `m1 = m1 + expr`; accumulator is set to the new cell value

### Removed

- Memory add/subtract commands `ma[1-9]` and `ms[1-9]` (replaced by the more general compound assignment directives)

## [0.2.0] - 2026-03-22

### Added

- Power operator `^` (right-associative, higher precedence than `*` and `/`), e.g. `2 ^ 10` → `1024`
- Modulo operator `%` (same precedence as `*` and `/`), e.g. `17 % 5` → `2`
- Constants `pi` and `e` usable in any expression, e.g. `sin(pi / 6)` → `0.5`
- Math functions: `sqrt`, `abs`, `floor`, `ceil`, `round`, `log` (base 10), `ln`, `exp`, `sin`, `cos`, `tan`
- Partial expressions now also accept `^` and `%` as leading operators
- New AST nodes: `Expr::Call(name, arg)`, `Op::Pow`, `Op::Mod`
- New `Ident(String)` token in the lexer — architectural prerequisite for functions and constants
- 38 new unit tests covering all new operators, constants, functions, precedence, and edge cases

## [0.1.0+004] - 2026-03-09

### Added

- CLI flag `-h` / `--help` — prints full usage reference with examples
- Unknown CLI flags now print an error message and exit with code 1

## [0.1.0+003] - 2026-03-09

### Added

- Memory cells `m1`–`m9` for storing intermediate values
- Store accumulator into cell: `m[1-9]` (standalone)
- Store expression result into cell: `expr m[1-9]` (trailing directive)
- Recall cell value inside any expression: `m[1-9]` used as an operand
- Add to cell: `ma[1-9]` (standalone) and `expr ma[1-9]` (trailing); prints new cell value
- Subtract from cell: `ms[1-9]` (standalone) and `expr ms[1-9]` (trailing); prints new cell value
- Show all non-zero cells: command `m`
- Clear all cells: command `mc`
- Clear a specific cell: `mc[1-9]`
- Expression display when memory refs are expanded (e.g. `m1 + 8 + m1` → prints `6 + 8 + 6`)
- New module `memory` encapsulating `Memory` struct, standalone command parser, directive extractor, and ref expander
- 26 unit tests covering all memory operations and parsing rules

## [0.1.0+002] - 2026-03-09

### Added

- CLI flag `-v` / `--version` — prints program version and exits

## [0.1.0+001] - 2026-03-07

### Added

- CLI calculator REPL with prompt `[result]:` acting as a numeric display
- Arithmetic operations: `+`, `-`, `*`, `/` with correct operator precedence
- Parenthesized expressions support, e.g. `(3 + 3) * 2`
- Partial expressions: input starting with an operator uses the current accumulator as the left operand (e.g. `+ 2`, `* 100`)
- Unary minus support (e.g. `-5`, `-(3 + 2)`)
- Command `c` — resets the accumulator to 0
- Command `cls` — clears the console screen
- Command `q` — exits the program
- Smart number formatting: integers displayed without decimal point; floats trimmed to 10 significant fractional digits with trailing zeros removed
- Module structure: `repl` (I/O loop), `parser` (tokenizer + recursive descent parser), `eval` (AST types + evaluator)
- 18 unit tests covering eval, formatting, parsing, operator precedence, parentheses, error cases, and partial-expression detection
