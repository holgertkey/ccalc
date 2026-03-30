# Octave Compatibility Roadmap

ccalc aims for maximum practical compatibility with Octave/MATLAB `.m` files.
The work is divided into phases in order of architectural dependency.

## Phase summary

| Phase | Goal | Status |
|---|---|---|
| 1 | Variables and assignment (`x = 5`, `who`, `clear`, `ws`/`wl`) | ✅ Done |
| 2 | Multi-argument functions (`atan2`, `mod`, `max`, `min`) | Planned |
| 3 | Matrix literals (`[1 2 3]`, `[1; 2; 3]`) | Planned |
| 4 | Matrix operations (`A * B`, `A'`, `A .* B`) | Planned |
| 5 | Range operator (`1:5`, `1:2:10`, `linspace`) | Planned |
| 6 | Indexing (`A(1,1)`, `v(2:4)`) | Planned |
| 7 | Comparison and logical operators (`==`, `~=`, `&&`) | Planned |
| 8 | Control flow (`if`, `for`, `while` in `.m` files) | Planned |
| 9 | User-defined functions (`function y = f(x) … end`) | Planned |
| 10 | String data types (`'char array'`, `"string object"`) | Planned |

## Key architectural decisions

**Phase 1** introduced `Env` (`HashMap<String, f64>`) and `Stmt`
(assignment vs expression), which are load-bearing for every subsequent phase.

**Phase 2** migrates `Expr::Call(String, Box<Expr>)` to
`Expr::Call(String, Vec<Expr>)` and introduces a function registry
(`HashMap<&str, BuiltinFn>`). The evaluator will no longer hard-code function
names.

**Phase 3** adds `ndarray` and a `Value` enum (`Scalar(f64)` | `Matrix(...)`),
migrating `Env` from `f64` to `Value`.

**Phase 6** resolves the syntactic ambiguity between `f(x)` (function call)
and `A(i)` (matrix indexing) by checking `Env` at eval time.

**Phase 8** adds multi-line input buffering to the REPL for unclosed
`if`/`for`/`while`/`end` blocks.

**Phase 10** adds two distinct string types following the modern MATLAB
standard: `'text'` (char array, numeric-compatible) and `"text"` (string
object).

## Compatibility notes

- `%` is a **comment** character (Octave/MATLAB convention). It terminates
  tokenization at that point. This is already implemented.
- `ans` is the sole implicit variable (Octave/MATLAB convention). The old
  accumulator (`acc`) and memory cells (`m1`–`m9`) were removed in Phase 1.
- 1-based indexing for matrices (Octave/MATLAB convention) — implemented in Phase 6.
- Where MATLAB and Octave differ, ccalc follows the **modern MATLAB standard
  (R2016b+)**.
- Full toolbox compatibility (Signal Processing, Optimization, etc.) is
  out of scope.
