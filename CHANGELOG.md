# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.7.0+002] - 2026-03-28

### Changed

- **Variable system** — replaced fixed memory cells (`m1`–`m9`) with a full named-variable environment:
  - `x = expr` assignment syntax (any valid identifier)
  - `ans` replaces `acc` as the implicit result of the last expression (Octave/MATLAB convention)
  - `who` lists all defined variables (replaces `m`)
  - `clear` / `clear x` clears all variables or a single one (replaces `mc` / `mc1`)
  - `ws` / `wl` save/load the workspace (replaces `ms` / `ml`)
  - `c` resets `ans` to `0` (behavior unchanged)
- **Engine restructure** — `memory.rs` removed; new `env.rs` module provides `Env` type (`HashMap<String, f64>`), workspace I/O, and identifier validation

## [0.7.0+001] - 2026-03-28

### Changed

- **Cargo workspace** — project restructured into two crates:
  - `crates/ccalc-engine` — new library crate containing the parser, evaluator, and memory modules; serves as the foundation for the upcoming Octave/MATLAB compatibility layer
  - `crates/ccalc` — binary crate (CLI), now depends on `ccalc-engine`
- **Single version source** — version is now defined once in `[workspace.package]` and inherited by both crates via `version.workspace = true`

### Added

- **mdBook documentation skeleton** — `docs/` directory with `book.toml` and `src/SUMMARY.md`; sections: User Guide, Architecture, Octave Compatibility

## [0.7.0+009] - 2026-03-26

### Added

- **Comments in pipe/file mode** — lines starting with `#` are skipped; inline `#` trims the rest of the line:
  ```
  # full-line comment
  10 * 5  # inline comment — the expression still evaluates
  ```
- **Semicolon suppression** — a trailing `;` evaluates the expression and updates the accumulator but prints nothing:
  ```
  0.06 / 12;   # silent intermediate step
  m1;
  1 + m1;      # still updates accumulator
  print "Monthly payment ($):"
  ```
- **`print` command** — explicit output control in pipe/file mode:
  - `print` — prints the current accumulator value
  - `print "label"` — prints `label value` (the label is the full quoted string, including any `:` the user writes)
- **Section headers** — `print "label"` after a blank line (or at the start) prints the label only, without the value, acting as a section separator:
  ```
  print "=== Results ==="

  10 + 5
  print "Sum:"
  ```
  Output:
  ```
  === Results ===
  15
  Sum: 15
  ```
- **`examples/` directory** — four annotated formula files demonstrating comments, `;`, and `print`:
  - `cylinder.ccalc` — volume and surface area of a cylinder
  - `mortgage.ccalc` — monthly mortgage payment
  - `data_storage.ccalc` — storage unit conversion (real GiB in a "500 GB" drive)
  - `resistors.ccalc` — Ohm's law: series, parallel, voltage divider, power

### Fixed

- Compound memory directives (`2 + 2 + 2 m1-`) now display the evaluated RHS value instead of the raw expression string:
  was: `10 - (2 + 2 + 2)` → now: `10 - 6`

## [0.7.0+003]

### Changed

- Compound memory directives (`m1+`, `m2*`, etc.) now display the full operation before the result:
  `80 m1-` with m1=850 prints `850 - 80` then `[ 770 ]:`
- Operation order is `cell op expr` to match the actual computation
- Multi-token expressions on the right-hand side are wrapped in parentheses:
  `2 * 10 + 2 + 2 m1+` with m1=10 prints `10 + (2 * 10 + 2 + 2)` then `[ 34 ]:`

## [0.7.0] - 2026-03-25

### Added

- Memory persistence: `ms` saves all non-zero memory cells to `~/.config/ccalc/memory.toml`; `ml` loads them back (clears all cells first, then restores from file)
- `dirs` dependency for cross-platform config directory resolution
- Expression conversion display: when the current base is non-decimal and the expression
  contains literals in other bases, the expression is printed with all values converted
  to the accumulator's base before the result
  - `[ 0b110 ]: 2 + 0b110 + 0xa` prints `0b10 + 0b110 + 0b1010` then `[ 0b10010 ]:`
  - `[ 0x6 ]: 0b11 + 0b11` prints `0x3 + 0x3` then `[ 0x6 ]:`

### Fixed

- `base` command and `expr base` suffix now display hex with `0x` prefix (e.g. `0xA` instead of `A`)
- Hex display in expression conversion now uses `0x` prefix, consistent with bin (`0b`) and oct (`0o`)

## [0.6.0] - 2026-03-25

### Added

- Percentage postfix operator: `N%` evaluates to `N * accumulator / 100`
  - `[1500]: 20%` → `300`
  - `[1500]: + 20%` → `1800` (add 20% of accumulator)
  - `[1800]: - 10%` → `1620` (subtract 10% of accumulator)
  - Disambiguated from modulo by lookahead: `17 % 5` still means modulo
- Implicit multiplication: a number or `)` immediately before `(` multiplies without an explicit `*`
  - `2(3 + 1)` → `8`
  - `(2+1)(4-1)` → `9`

## [0.5.0] - 2026-03-24

### Added

- Hex, binary, and octal input literals: `0xFF`, `0b1010`, `0o17` — parsed directly in expressions
- Display base commands: `hex`, `dec`, `bin`, `oct` — change how all subsequent results are shown (including the prompt)
- Inline base suffix: `0xFF + 0b1010 hex` evaluates the expression and switches the display base in one step
- `base` command — prints the current accumulator value in all four bases simultaneously
- Configurable decimal precision: `p` shows current precision, `p<N>` sets it (0–15 decimal places, default 10)
- Scientific notation display for very large (`|n| >= 1e15`) and very small (`|n| < 1e-9`) numbers
- All new formatting and base commands work identically in REPL, pipe, and single-expression modes

### Changed

- `memory.display_nonzero` now accepts a format closure, allowing memory cells to be printed in the current display base

## [0.4.0] - 2026-03-23

### Added

- Pipe / non-interactive mode: when stdin is not a terminal, ccalc runs silently (no prompt, one result per line)
- Single-expression argument mode: `ccalc "expr"` evaluates and prints the result, exits with code 1 on error
- File redirect support: `ccalc < formulas.txt` (handled by the same pipe path)
- Accumulator carries over across lines in pipe mode — multi-step calculations work naturally
- Commands `q`, `c`, `mc`, `mc[1-9]`, `m[1-9]` all work in pipe mode; `cls` and `m` are silently ignored

### Changed

- Refactored `repl.rs`: extracted shared `evaluate()` / `evaluate_expanded()` / `apply_compound()` helpers used by all three modes
- `main.rs` now detects mode via `std::io::IsTerminal` (no extra dependency)

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
