# Phase 30 — Colormaps & imagesc

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
  from the active colormap LUT. Requires `plot-svg`.
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

## See also

- [Plot functions guide](../guide/plot.md)
- [Phase 29 — Plot Engine](./phase29-plot.md)
- [Plugins guide](../guide/plugins.md)
