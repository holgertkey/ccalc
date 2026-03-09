# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

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
