# Octave Compatibility

Implementation history of ccalc's Octave/MATLAB compatibility, organized by
development phase. Each phase documents what was added, the design decisions
made, and the test coverage.

## Phase summary

| Phase | Feature area | Version |
|---|---|---|
| [Roadmap](./roadmap.md) | Future work and open design questions | — |
| [1](./phase1-variables.md) | Named variables | v0.1.0 |
| [2](./phase2-functions.md) | Multi-argument functions | v0.7.0 |
| [3](./phase3-matrices.md) | Matrix literals | v0.8.0 |
| [4](./phase4-matrix-ops.md) | Matrix operations (`*`, `'`, `.*`) | v0.9.0 |
| [5](./phase5-range.md) | Range operator (`1:5`, `0:0.1:1`) | v0.10.0 |
| [6](./phase6-indexing.md) | Indexing (`A(i,j)`, `v(:)`) | v0.11.0 |
| [7](./phase7-logic.md) | Comparison and logical operators | v0.11.0 |
| [7.5](./phase75-vector-utils.md) | Vector utilities, `end`, special constants | v0.11.0 |
| [8](./phase8-complex.md) | Complex numbers | v0.12.0 |
| [9](./phase9-strings.md) | String data types | v0.13.0 |
| [10](./phase10-io.md) | C-style I/O and `format` | v0.14.0 |
| [10.5](./phase105-fileio.md) | File I/O and filesystem queries | v0.14.0 |
| [11](./phase11-control-flow.md) | Core control flow | v0.15.0 |
| [11.5](./phase115-extended-control-flow.md) | Extended control flow, `run`/`source` | v0.16.0 |
| [12](./phase12-user-functions.md) | User-defined functions | v0.17.0 |
| [12.5](./phase125-cell-arrays.md) | Cell arrays | v0.17.0 |
| [12.6](./phase126-language-polish.md) | Language polish | v0.18.0 |
| [13](./phase13-structs.md) | Structs | v0.19.0 |
| [13.5](./phase135-struct-arrays.md) | Struct arrays | v0.19.0 |
| [13.6](./phase136-backslash-path.md) | Backslash operator and path system | v0.20.0 |
| [14](./phase14-error-handling.md) | Error handling | v0.20.0 |
| [15](./phase15-indexed-assignment.md) | Indexed assignment | v0.21.0 |
| [15.6](./phase156-scoping.md) | Variable scoping | v0.21.0 |
| [16](./phase16-packages.md) | Package namespaces | v0.21.0 |
| [17](./phase17-statistics.md) | Statistics and random numbers | v0.21.0 |
| [18](./phase18-linear-algebra.md) | Advanced linear algebra | v0.22.0 |
| [19](./phase19-repl-tooling.md) | REPL tooling | v0.23.0 |
| [20a](./phase20a-json.md) | JSON encode/decode | v0.24.0 |
| [20c](./phase20c-csv.md) | CSV improvements | v0.24.0 |
| [20.5](./phase205-mat.md) | MAT file read | v0.25.0 |
| [21](./phase21-string-regex.md) | String completions and regex | v0.26.0 |
| [22](./phase22-datetime.md) | Datetime and duration | v0.27.0 |
| [23](./phase23-set-operations.md) | Matrix utilities and set operations | v0.28.0 |
| [24](./phase24-polynomials.md) | Polynomial operations and interpolation | v0.29.0 |
| [25](./phase25-eval.md) | Dynamic evaluation and timing | v0.30.0 |
| [26](./phase26-fft.md) | FFT and signal processing | v0.31.0 |
| [27](./phase27-complex-matrices.md) | Complex matrices | v0.32.0 |
| [28](./phase28-plugins.md) | Plugin architecture | v0.33.0 |
| [29](./phase29-plot.md) | Plot engine | v0.34.0 |
| [30](./phase30-colormap.md) | Colormaps, `imagesc`, `surf`, style strings | v0.35.0 |
