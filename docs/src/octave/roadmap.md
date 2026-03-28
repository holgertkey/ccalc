# Octave Compatibility Roadmap

ccalc is being extended to support Octave/MATLAB syntax in the engine crate.
The work is divided into 9 phases in order of architectural importance.

## Phase summary

| Phase | Goal | Status |
|---|---|---|
| 1 | Variables and assignment (`x = 5`, `who`, `clear`) | Planned |
| 2 | Multi-argument functions (`atan2`, `mod`, `max`) | Planned |
| 3 | Matrix literals (`[1 2 3]`, `[1; 2; 3]`) | Planned |
| 4 | Matrix operations (`A * B`, `A'`, `A .* B`) | Planned |
| 5 | Range operator (`1:5`, `1:2:10`, `linspace`) | Planned |
| 6 | Indexing (`A(1,1)`, `v(2:4)`) | Planned |
| 7 | Comparison and logical operators (`==`, `~=`, `&&`) | Planned |
| 8 | Control flow (`if`, `for`, `while` in `.m` files) | Planned |
| 9 | User-defined functions (`function y = f(x) … end`) | Planned |

## Key architectural decisions

**Phase 1** introduces `Value` (replacing bare `f64`) and `Env` (variable
environment), which are load-bearing for every subsequent phase.

**Phase 3** adds `ndarray` as the first external computation dependency.

**Phase 6** resolves the syntactic ambiguity between `f(x)` (function call)
and `A(i)` (matrix indexing) by checking `Env` at parse/eval time.

**Phase 8** adds multi-line input buffering to the REPL for unclosed
`if`/`for`/`while`/`end` blocks.

## Compatibility notes

- `%` is already **modulo** in ccalc; in Octave `%` starts a comment.
  Resolution deferred to Phase 8 (scripting).
- The accumulator (`acc`) and memory cells (m1–m9) coexist with Octave
  variables throughout all phases.
- Rarely-used Octave features are left unimplemented until clearly needed.

See individual phase pages for detailed implementation plans.
