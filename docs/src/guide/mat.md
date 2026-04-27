# MAT Files

ccalc can read MATLAB Level 5/7 `.mat` files using `load`. This lets you
exchange data with MATLAB, Octave, SciPy, and any other tool that writes the
standard MAT format.

> **Requires the `mat` feature:**
> ```bash
> cargo build --release --features mat
> ```
> Without this flag, calling `load('*.mat')` returns an error message
> explaining how to enable the feature.

---

## Loading a MAT file

### Assignment form

`data = load('file.mat')` reads all variables from the file and returns
a `Struct` whose fields are the variable names.

```matlab
data = load('results.mat');

data.score        % scalar variable from the file
data.readings     % matrix variable from the file
data.label        % char-array variable from the file
data.sensor.id    % struct field — nested access works directly
```

Access the shape of a loaded matrix:

```matlab
fprintf('%dx%d\n', size(data.A, 1), size(data.A, 2))
```

### Bare form

`load('file.mat')` (without an assignment) injects all variables directly
into the current workspace:

```matlab
load('results.mat')

% All variables are now in scope:
score
readings
sensor.gain
```

This is equivalent to the assignment form followed by assigning each field
to a workspace variable.

---

## Type mapping

| MATLAB type | ccalc value |
|-------------|-------------|
| scalar `double` | `Scalar` |
| M×N `double` matrix | `Matrix` |
| `char` array (string) | `Str` (char array) |
| `struct` | `Struct` |
| struct array | `StructArray` |
| `cell` array | `Cell` |
| empty / null | `Scalar(NaN)` |

Complex and sparse matrices are not yet supported.

---

## Working with loaded data

### Scalar

```matlab
data = load('results.mat');
s = data.score;
fprintf('score = %g,  score^2 = %g\n', s, s^2)
```

### Matrix

```matlab
A = data.A;
fprintf('A is %dx%d\n', size(A, 1), size(A, 2))
fprintf('trace(A''*A) = %.1f\n', trace(A'*A))
```

### Char array

```matlab
lbl = data.label;
fprintf('label = %s\n', upper(lbl))
```

### Struct fields

```matlab
sen = data.sensor;
scaled = data.readings * sen.gain;
fprintf('scaled mean = %.2f\n', mean(scaled))
```

---

## Saving MAT files

Writing `.mat` files is not yet supported. `save('out.mat', ...)` returns
an informative error. Use `save` without a `.mat` extension (or `ws`) to
persist workspace variables in ccalc's native TOML format.

---

## Example

The `examples/mat/mat.calc` file demonstrates all MAT-file features:

```bash
cargo run --release --features mat -- examples/mat/mat.calc
```

It covers: assignment form, scalar arithmetic, row-vector statistics,
matrix algebra, char-array built-ins, struct field access, bare workspace
merge, and a simple signal-analysis routine.

### Generating the fixture

The example uses `examples/mat/fixtures/sample.mat`, which can be
regenerated with:

```bash
cargo test --features mat create_example_fixture -- --ignored
```
