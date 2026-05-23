# Phase 30 — Colormaps, imagesc & 3D Surfaces

Matrix-to-image rendering: false-colour heat-maps with configurable colormaps
and an optional colour-scale legend (colorbar). Builds on the `PlotPlugin`
infrastructure from Phase 29.

---

## Phase 30a — `colormap` + `imagesc` + `colorbar` (v0.37.0) ✅

### What's new

- **`imagesc(Z)`** — renders a numeric matrix as a false-colour ASCII image
  using 10 density characters. Requires the `plot` feature.
- **`imagesc(Z, 'file.svg')` / `imagesc(Z, 'file.png')`** — saves a
  full-colour heat-map to a file. One `Rectangle` per matrix cell, RGB colour
  from the active colormap LUT. Canvas size from `figure(w, h)` (default
  800 × 600 px). Requires `plot-svg`.
- **`colormap('name')`** — sets the active colormap, consumed by the next
  `imagesc` call. Validates against the list of supported names; returns an
  error for unknown names.
- **`colorbar()`** — sets a flag that tells the next file-export `imagesc` call
  to append an 80 px colour-scale strip with five tick labels. Silently ignored
  in ASCII mode.

### Supported colormaps

Eight colormaps implemented as 8-stop LUTs in `colormap.rs`, interpolated
linearly between stops via `lut_lerp`:

| Name | Description |
|---|---|
| `viridis` | Perceptually uniform, blue → green → yellow (default) |
| `inferno` | Black → purple → orange → white |
| `magma` | Black → purple → pink → white |
| `plasma` | Blue-purple → orange → yellow |
| `hot` | Black → red → yellow → white |
| `cool` | Cyan → magenta |
| `jet` | Classic MATLAB: blue → cyan → green → yellow → red |
| `gray` | Black → white (monochrome) |

**Accepted argument forms:**

| Call | Canvas | Feature |
|---|---|---|
| `imagesc(Z)` | — (ASCII) | `plot` |
| `imagesc(Z, path)` | from `figure(w,h)`, else 800 × 600 px | `plot-svg` |

### FigureState additions

```rust
pub colormap: Option<String>,  // active colormap name; None → "viridis"
pub colorbar: bool,            // draw colorbar strip in file export
```

Both fields are cleared (reset to defaults) after each `imagesc` render,
together with the existing annotation fields (`title`, `xlabel`, etc.).

### ASCII tier

`render_imagesc_ascii` in `colormap.rs` (gated `#[cfg(feature = "plot")]`):

1. Find `z_min` / `z_max` over all cells.
2. Map each cell to one of 10 density characters: `" .:-=+*#@█"`.
3. Print title (if set), then the character grid row by row.
4. `colormap` and `colorbar` annotations are silently ignored.

### File tier

`render_imagesc_file` in `colormap.rs` (gated `#[cfg(feature = "plot-svg")]`):

1. If `colorbar` is set, call `root.split_horizontally(w - CB_WIDTH)` to
   produce `(main_area, colorbar_area)`. Otherwise use the full canvas.
2. Call `draw_imagesc_cells` on `main_area`:
   - Scale each cell value to `[0.0, 1.0]`.
   - Map through `apply_colormap(t, name)` → `(u8, u8, u8)`.
   - Draw one `Rectangle` per cell; MATLAB row-order (row 0 = top-left) is
     preserved by mapping row `r` to y-range `[(nrows-1-r), (nrows-r)]`.
3. If `colorbar` is set, call `draw_colorbar` on `colorbar_area`:
   - Draw 200 thin horizontal rectangles from bottom (`z_min`) to top (`z_max`).
   - Add a right y-axis with 5 tick labels.

### Implementation

| Source file | Role |
|---|---|
| `crates/ccalc-plot/src/colormap.rs` | LUT data, `apply_colormap`, ASCII + file render |
| `crates/ccalc-plot/src/dispatch.rs` | `extract_matrix` helper (returns flat `Vec<f64>` + dims) |
| `crates/ccalc-plot/src/lib.rs` | `FigureState` fields; match arms for `colormap`/`colorbar`/`imagesc` |

`extract_matrix` returns a plain `Vec<f64>` with `(nrows, ncols)` so that
`colormap.rs` never needs to name the `ndarray` type directly.

### Tests

**Unit tests in `lib.rs`** (12 tests):

- `colormap("viridis")` sets `FigureState.colormap`.
- `colormap("unknown")` returns an error naming the valid options.
- `colorbar()` sets `FigureState.colorbar = true`.
- `imagesc` with a non-matrix argument returns an error.
- `imagesc` with no arguments returns an error.
- `imagesc` returns `Void` (no feature builds).
- ASCII `imagesc` completes without error (with `plot` feature).
- ASCII `imagesc` with colorbar completes without error.
- Gray colormap extremes return black / white exactly.

**Integration tests in `svg_png_tests.rs`** (3 tests, gated `plot-svg`):

- `imagesc(magic(8), 'heat.svg')` → file contains `<svg`.
- `imagesc(magic(8), 'heat.png')` → PNG magic bytes `\x89PNG`.
- `imagesc` with `colorbar()` + `colormap("jet")` → SVG file created.

### Example scripts

- `examples/colormap/imagesc_demo.calc` — gradient matrix + all 8 colormaps + colorbar
- `examples/colormap/mandelbrot.calc` — Mandelbrot escape-count map with `colormap('inferno')`
- `examples/colormap/julia.calc` — Julia set with `colormap('magma')`

---

## Phase 30b — `meshgrid` + `surf` + `mesh` (v0.37.0+001) ✅

3D surface visualisation: `surf` draws a colored surface, `mesh` draws a
wireframe. Both require `meshgrid` to generate the coordinate matrices.

### `meshgrid` — engine change

`meshgrid` is a new engine built-in (added to `builtin_names()` and
`call_builtin` in `eval.rs`).  Uses `NARGOUT` to select single or multi-output:

| Call | Returns |
|---|---|
| `[X, Y] = meshgrid(x, y)` | `Value::Tuple([X_mat, Y_mat])` |
| `X = meshgrid(x, y)` | `Value::Matrix(X_mat)` (X only) |
| `[X, Y] = meshgrid(x)` | square N×N grid (x used for both axes) |

X is M×N where every row is a copy of `x`; Y is M×N where every column is a
copy of `y`.  The `1`-argument form uses `x` for both dimensions (MATLAB
compatible).

### `surf` and `mesh` — plot plugin

Both functions are dispatched by the `PlotPlugin` (added to `EXPORTED`).
Argument forms:

| Call | Output |
|---|---|
| `surf(X, Y, Z)` | ASCII elevation map (requires `plot` feature) |
| `surf(X, Y, Z, 'f.svg')` | SVG file (requires `plot-svg`) |
| `surf(X, Y, Z, 'f.png')` | PNG file (requires `plot-svg`) |
| `mesh(X, Y, Z)` | wireframe ASCII (same as surf in ASCII mode) |
| `mesh(X, Y, Z, 'f.svg')` | wireframe SVG |

X, Y, Z must all have the same dimensions (M×N). A clear error is returned
if dimensions differ.

### ASCII tier

`render_surf_ascii` in `surface.rs` (gated `#[cfg(feature = "plot")]`):

1. Compute the maximum Z over each column (`col_max`).
2. Print a character grid of height 20: row `k` prints `#` for columns where
   `col_max[c] ≥ z_min + z_range * (k / 20)`.
3. Print x-axis tick labels (first and last x value).
4. Print `xlabel` / `ylabel` / `zlabel` footer lines when set.

Both `surf` and `mesh` produce identical ASCII output.

### File tier

`draw_surface` in `surface.rs` (gated `#[cfg(feature = "plot-svg")]`).

**Axis mapping** — chart `(X, Y, Z)` = our `(X, Z_height, Y_depth)`:

| Chart dim | plotters role | our value |
|---|---|---|
| First (X) | horizontal left–right | `x_vals` |
| Second (Y) | visual height (up) | `z` values |
| Third (Z) | depth (into page) | `y_vals` |

Points: `(x_vals[c], z[r*nc+c], y_vals[r])` ensure our Z (function value)
is the visual height and our Y (spatial coordinate) is depth.
This matches the conventional MATLAB `surf` view.

**`surf`**: draws all row lines _and_ all column lines, each colored by the
mean Z of that row or column through the active colormap.

**`mesh`**: draws only row lines (sparser wireframe appearance).

Note: `SurfaceSeries` was evaluated but rejected — its axis-mapping convention
(`(xi, yi, f(xi,yi))` → `(chart_X, chart_Y_height, chart_Z_depth)`) placed our
spatial Y values on the height axis, causing a flat-wall artifact.  `LineSeries`
with explicit point ordering is simpler and correct.

### Implementation

| Source file | Role |
|---|---|
| `crates/ccalc-engine/src/eval.rs` | `meshgrid` cases in `call_builtin`; entry in `builtin_names()` |
| `crates/ccalc-plot/src/surface.rs` | ASCII + SVG/PNG renderers for `surf` and `mesh` |
| `crates/ccalc-plot/src/lib.rs` | `surf`/`mesh` in `EXPORTED`; dispatch to `render_surface` |

### Tests

**Engine tests** (`eval_tests.rs`, mod `phase30b_tests`): 5 tests —
`meshgrid` dimensions, X row equality, Y column equality, single-output form,
single-argument square form.

**Plot tests** (`lib.rs`, mod `tests`): 7 tests —
missing arguments error, dimension mismatch error (surf + mesh), ASCII no-error
(surf + mesh), SVG file creation (`surf`), PNG magic bytes (`mesh`).

### Example scripts

- `examples/surf_demo/surf_demo.calc` — sine wave surface + Gaussian bell
- `examples/surf_demo/mesh_demo.calc` — sine wave wireframe + saddle surface

Both write output files to `examples/surf_demo/tmp/`.

---

## Phase 30c — `contour` + `contourf` (v0.37.0+002) ✅

2D contour plots using the marching squares algorithm.

### Functions

| Call | Output |
|---|---|
| `contour(X, Y, Z)` | ASCII char-art isolines (requires `plot`) |
| `contour(X, Y, Z, n)` | ASCII with `n` levels |
| `contour(X, Y, Z, n, 'f.svg')` | SVG isoline chart (requires `plot-svg`) |
| `contour(X, Y, Z, n, 'f.png')` | PNG isoline chart |
| `contourf(X, Y, Z, n, 'f.png')` | PNG filled-contour chart |

X, Y, Z are M×N matrices from `meshgrid`. Default level count is 10.

### Algorithm

**`compute_levels(z_min, z_max, n)`** — returns `n` interior levels evenly spaced
inside `(z_min, z_max)` at positions `z_min + (z_max − z_min) × k / (n+1)` for
`k = 1..=n`. Levels never equal the data extrema.

**`marching_squares`** — 16-case lookup table over every 2×2 cell.
Bit assignment: bit 0 = BL (`z[r][c]`), bit 1 = BR, bit 2 = TR, bit 3 = TL.
Edge crossings use linear interpolation. Saddle cases 5 and 10 split into two
separate islands (no centre-value disambiguation).

### ASCII tier

`render_contour_ascii` (gated `#[cfg(feature = "plot")]`):
80 × 24 char grid. Each character is chosen from `" .:-=+*#"` by the Z band of
the sampled cell (band 0 = lowest = space, band 7 = highest = `#`).

### File tier

`draw_contour` (gated `#[cfg(feature = "plot-svg")]`), called by both
`render_contour_file` and `render_contourf_file`:

1. Build a `ChartBuilder` with the actual data coordinate range (`x_lo..x_hi`,
   `y_lo..y_hi`).
2. If `filled`: draw one `Rectangle` per grid cell, colored by the cell's mean Z
   mapped through the active colormap. Band index = count of levels ≤ `z_mean`;
   normalised `t = band / n_levels`.
3. Draw one `LineSeries` per marching-squares segment, colored by level index
   through the colormap.

### Bug fix (v0.37.0+003)

A parser precedence bug caused `-X .^ 2` to be evaluated as `(-X) .^ 2 = X^2`
instead of `-(X .^ 2) = -X^2`. This made `exp(-X .^ 2 - Y .^ 2)` compute
`exp(X^2 + Y^2)` — inverted — so contour plots of the Gaussian bell showed peaks
at the corners rather than the centre.

Fix: reordered the recursive-descent parse chain so unary minus has lower
precedence than `^`/`.^`, matching MATLAB/Octave semantics.

### Implementation

| Source file | Role |
|---|---|
| `crates/ccalc-plot/src/contour.rs` | `compute_levels`, `marching_squares`, ASCII + file renderers |
| `crates/ccalc-plot/src/lib.rs` | `contour`/`contourf` in `EXPORTED`; dispatch to `render_contour` |
| `crates/ccalc-engine/src/parser.rs` | Precedence fix: `parse_term → parse_unary → parse_power → parse_primary` |
| `crates/ccalc-engine/src/parser_tests.rs` | Regression test `test_unary_minus_lower_precedence_than_power` |

### Tests

**Unit tests in `contour.rs`** (4 tests): `compute_levels` zero/one/five counts;
marching squares case 1, no-crossing, saddle case 5, too-small grid.

**Plot tests in `lib.rs`** (7 tests): missing args, dimension mismatch, wrong
level type (contour + contourf), ASCII no-error, SVG file creation, PNG magic
bytes.

### Example

`examples/contour_demo/contour_demo.calc` — Gaussian bell + saddle surface;
writes four files to `examples/contour_demo/tmp/`.

---

---

## Phase 30.5 — Unified color system (v0.41.0) ✅

Closes the color gaps: `colormap()` gains custom matrix input; style strings
gain full color names, hex codes, and RGB matrices; `bar`/`stem`/`hist`/`quiver`
gain a `'color'` named argument. All plot types now share a consistent two-layer
color model.

### Phase 30.5a — `ColormapSpec` enum + `colormap(M)` N×3 matrix input (v0.41.0) ✅

**Goal:** `colormap()` accepts a custom N×3 matrix (rows = control points,
columns = R G B in [0, 1]) in addition to named colormaps.

**New types in `colormap.rs`:**

```rust
pub enum ColormapSpec {
    Named(String),               // one of 8 built-in names
    Custom(Vec<(u8, u8, u8)>),   // user-supplied control points, 0–255
}

pub fn apply_colormap_spec(t: f64, spec: &ColormapSpec) -> (u8, u8, u8)
pub fn validate_colormap_spec(spec: &ColormapSpec) -> Result<(), String>
```

`FigureState.colormap` changed from `Option<String>` → `Option<ColormapSpec>`.

**Engine dispatch (`eval.rs`):**

```matlab
colormap([0 0 1; 1 0 0])        % two-stop blue → red
colormap([0 0 1; 1 1 0; 1 0 0]) % three-stop blue → yellow → red
```

The matrix must be N×3 with values in [0, 1]; a 1-row matrix returns an error
("custom colormap must have at least 2 rows").

**Tests:** 6 tests — custom 2-point, 3-point, too-short LUT, Named→`apply_colormap`
parity, matrix dispatch via engine, wrong column count.

---

### Phase 30.5b — Extended style strings: full names, hex, 1×3 RGB matrix (v0.41.0+001) ✅

**Goal:** `plot(x, y, 'red')`, `plot(x, y, '#FF4400')`, and
`plot(x, y, [1 0.27 0])` all work. Same extensions reach `bar`, `stem`,
`scatter`, `fill`, `area`.

**`parse_color_token` in `style.rs`** — central color parser used by both
`parse_style_str` and the `'color'` named-argument handler:

| Input | Example | Resolves to |
|---|---|---|
| Single letter | `'r'` | `StyleColor(255,0,0)` |
| Full name | `'orange'` | `StyleColor(255,165,0)` |
| `gray`/`grey` | both spellings | `StyleColor(128,128,128)` |
| Hex `#RRGGBB` | `'#FF4400'` | `StyleColor(255,68,0)` |

Full names supported: `red`, `green`, `blue`, `cyan`, `magenta`, `yellow`,
`black`, `white`, `orange`, `purple`, `gray`/`grey`.

**1×3 RGB matrix detection** in `extract_style_and_file_arg` (dispatch.rs):
a trailing `[r g b]` matrix with all values ∈ [0, 1] is consumed as a
`StyleSpec { color: Some(StyleColor) }`.

**`'color'` named argument** — trailing `('color', <value>)` pair in arg list
builds a minimal `StyleSpec`. The value can be a string or a 1×3 RGB matrix.

```matlab
bar(x, y, 'color', 'red')
hist(v, 20, 'color', '#FF8800')
bar(v, 'color', [0.2 0.6 1.0])
```

**Tests:** 8 tests — full name red/orange, gray/grey alias, hex parse, bad hex
format, RGB matrix dispatch, `'color'` named arg for bar (ASCII), `'color'` hex
via named arg.

---

### Phase 30.5c — `Option<StyleSpec>` for Bar / Stem / Hist / Quiver (v0.41.0+002) ✅

**Goal:** all `PendingSeries` variants carry an `Option<StyleSpec>`; both the
accumulating (`draw_panel`) and standalone render paths use the color when
present, falling back to `SERIES_COLORS[i % 7]`.

**`PendingSeries` enum changes (`lib.rs`):**

```rust
Bar(Vec<f64>, Vec<f64>, Option<StyleSpec>),
Stem(Vec<f64>, Vec<f64>, Option<StyleSpec>),
Hist { counts: Vec<usize>, edges: Vec<f64>, style: Option<StyleSpec> },
Quiver(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Option<StyleSpec>),
```

**New dispatch helpers in `lib.rs`:**

- `render_bar_xy(x, y, path, style, state)` — dispatches bar to ASCII or file
- `render_stem_xy(x, y, path, style, state)` — dispatches stem to ASCII or file

**`extract_style_and_file_arg_min(args, min_data)` in `dispatch.rs`:**
Variant of the style extractor with a configurable guard: the 1×3 RGB matrix
detection only fires when `args.len() > min_data`. Quiver uses `min_data = 4`
to prevent a data vector from being mistaken for a color spec.

**Color resolution in `file.rs` (`draw_panel`):**

```rust
PendingSeries::Bar(xs, ys, style) => {
    let color = style_to_rgb(style)
        .unwrap_or(SERIES_COLORS[series_idx % SERIES_COLORS.len()]);
    // draw rectangles with color
}
```

Same pattern applied to `Stem`, `Hist`, and `Quiver` arms.

**Tests:** 6 tests — bar red, bar default color cycle, stem blue, hist hex
orange, quiver green, structural exhaustiveness check.

---

### Two-layer color model summary

```
┌─────────────────────────────────────────────────────────┐
│  Discrete layer   (per-series)                          │
│    StyleColor(r,g,b)  ← style string / RGB matrix /    │
│                          'color' named arg               │
│    Fallback: SERIES_COLORS[i % 7]                       │
├─────────────────────────────────────────────────────────┤
│  Continuous layer (per-value)                           │
│    ColormapSpec::Named(s)  → apply_colormap(t, s)       │
│    ColormapSpec::Custom(v) → lut_lerp(t, &v)            │
│    Output: (u8,u8,u8) for imagesc/surf/mesh/contour     │
└─────────────────────────────────────────────────────────┘
```

The layers are independent: a scatter series can carry its own `StyleColor`
while an `imagesc` underneath uses `ColormapSpec`.

---

## See also

- [Plot functions guide](../guide/plot.md)
- [Phase 29 — Plot Engine](./phase29-plot.md)
- [Plugins guide](../guide/plugins.md)
