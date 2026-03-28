# ccalc

`ccalc` is a command-line calculator with an accumulator, memory cells,
multi-base arithmetic, and script file support.

## Quick start

```sh
# Interactive REPL
ccalc

# Single expression
ccalc "2 ^ 32"

# Pipe mode
echo "sqrt(2)" | ccalc
```

## Project structure

| Crate | Role |
|---|---|
| `crates/ccalc` | CLI binary: argument parsing, REPL, pipe mode |
| `crates/ccalc-engine` | Library: tokenizer, parser, AST evaluator, memory cells |

The engine crate is the foundation for the upcoming Octave/MATLAB
compatibility layer.

## Source

- Repository: <https://github.com/holgertkey/ccalc>
- Changelog: see `CHANGELOG.md` in the repository root
