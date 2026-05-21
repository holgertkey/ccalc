# User Guide

This guide covers everything you need to use ccalc effectively: from the first
expression in the REPL to scripts with functions, matrices, structs, and plots.

## Contents

| Topic | What you will find |
|---|---|
| [Getting Started](./getting-started.md) | Installation, first session, key concepts |
| [REPL Mode](./repl.md) | Interactive session: history, tab completion, workspace |
| [Pipe & Script Mode](./pipe-mode.md) | One-liners, shell pipelines, running `.m` files |
| [Arithmetic & Operators](./arithmetic.md) | Precedence, bitwise ops, the `ans` variable |
| [Variables](./variables.md) | Assignment, `who`, `clear`, workspace save/load |
| [Number Bases](./number-bases.md) | Hex `0x`, binary `0b`, octal `0o` input and display |
| [Number Display Format](./precision.md) | `format short/long/rat/hex` and custom precision |
| [Formatted Output](./formatted-output.md) | `fprintf`, `sprintf`, `%d/%f/%g/%s` specifiers |
| [Configuration](./configuration.md) | `~/.config/ccalc/config.toml` reference |
| [Matrices](./matrices.md) | Literals, arithmetic, indexing, built-in constructors |
| [Vector & Data Utilities](./vectors.md) | `sum`, `sort`, `find`, `reshape`, `unique`, … |
| [Comparison & Logical Operators](./logic.md) | `==`, `~=`, `&&`, `\|`, element-wise ops |
| [Complex Numbers](./complex.md) | `3+4i`, `abs`, `angle`, `conj`, complex matrices |
| [Strings](./strings.md) | Char arrays, string objects, built-in string functions |
| [File I/O](./file-io.md) | `fopen/fclose`, `dlmread/dlmwrite`, `isfile` |
| [Control Flow](./control-flow.md) | `if`, `for`, `while`, `switch`, `break`, `continue` |
| [User-defined Functions](./user-functions.md) | Named functions, lambdas, `nargin`/`nargout` |
| [Cell Arrays](./cell-arrays.md) | `{...}`, brace indexing, `cellfun`, `arrayfun` |
| [Structs and Struct Arrays](./structs.md) | `.field` access, `struct(...)`, `fieldnames` |
| [Error Handling](./error-handling.md) | `error`, `try/catch`, `pcall`, `lasterr` |
| [Variable Scoping](./scoping.md) | `global`, `persistent`, `private/` directories |
| [Statistics & Random Numbers](./statistics.md) | `mean`, `std`, `rand`, `randn`, distributions |
| [Linear Algebra](./linear-algebra.md) | `eig`, `svd`, `lu`, `qr`, `chol`, `pinv` |
| [JSON](./json.md) | `jsondecode`, `jsonencode` |
| [CSV — Tables and Matrices](./csv.md) | `readtable`, `writetable`, `csvread`, `csvwrite` |
| [MAT Files](./mat.md) | `load`/`save` with `.mat` format |
| [Datetime & Duration](./datetime.md) | `datetime`, `duration`, formatting, arithmetic |
| [Matrix Utilities & Set Operations](./set-operations.md) | `intersect`, `union`, `ismember`, `kron` |
| [Polynomial Operations & Interpolation](./polynomials.md) | `polyval`, `polyfit`, `roots`, `interp1` |
| [FFT & Signal Processing](./fft.md) | `fft`, `ifft`, `fftshift`, `freqz` |
| [Dynamic Evaluation & Timing](./eval.md) | `eval`, `feval`, `tic`/`toc` |
| [Plugins](./plugins.md) | The `Plugin` trait and custom built-ins |
| [Plot Functions](./plot.md) | `plot`, `scatter`, `surf`, `contour`, `subplot`, … |
