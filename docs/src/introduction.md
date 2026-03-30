# ccalc

A fast terminal calculator with Octave/MATLAB syntax and script support —
one binary, no runtime.

Octave is hundreds of megabytes. Python requires a runtime. ccalc is a
single self-contained binary that starts instantly and works anywhere:
interactive sessions, shell scripts, CI pipelines, Docker containers.

## Quick start

```sh
# Interactive REPL
ccalc

# Single expression
ccalc "2 ^ 32"

# Script file
ccalc script.m

# Pipe mode
echo "sqrt(2)" | ccalc
```

## Who is it for?

| User | Typical use |
|------|-------------|
| Embedded / systems engineer | Arithmetic, hex/bin conversions, bit masks |
| DevOps / SRE | Quick calculations in scripts and pipelines |
| Scientist / student | Interactive session with variables and math functions |
| MATLAB / Octave user | Familiar syntax, no heavy installation |

## Project structure

| Crate | Role |
|---|---|
| `crates/ccalc` | CLI binary: argument parsing, REPL, pipe mode |
| `crates/ccalc-engine` | Library: tokenizer, parser, AST evaluator, variable environment |

The engine crate is the computation foundation. It has no I/O dependencies
and is the target for all Octave/MATLAB compatibility work (Phases 1–10).

## Compatibility standard

Where MATLAB and Octave differ, ccalc follows the **modern MATLAB standard
(R2016b+)**. See [Architecture → Overview](./architecture/overview.md) for
design principles.

## Source

- Repository: <https://github.com/holgertkey/ccalc>
- Changelog: see `CHANGELOG.md` in the repository root
