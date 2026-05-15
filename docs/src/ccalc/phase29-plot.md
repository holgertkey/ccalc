# Phase 29 — Plot Engine

Fills `PlotPlugin` with real rendering logic, building on the plugin
infrastructure added in Phase 28.

---

## Phase 29a — ASCII terminal rendering (v0.35.0) ✅

### What's new

- **`plot(x, y)`** — connected line chart via `textplots 0.8` Braille canvas;
  requires the `plot` Cargo feature.
- **`scatter(x, y)`** — point-cloud chart using the same renderer.
- **`xlabel("text")`, `ylabel("text")`, `title("text")`** — annotate the next
  plot; state is consumed and reset after each render.
- **`FigureState`** — thread-local struct that accumulates annotation state
  between calls.
- **`Plugin::call` signature** — `name: &str` parameter added as the first
  argument so a single plugin instance can dispatch multiple exported names.
  Existing plugin implementations must add `_name: &str` as their first
  parameter.
- **`ccalc-plot` feature flags** — `plot` (textplots/ASCII), `plot-svg`
  (plotters/SVG+PNG), `plot-all` (both).

### Rendering notes

`textplots 0.8` renders data using Braille characters (U+2800–U+28FF).
The canvas is populated only via the method-chain call
`.lineplot(&data).display()`. Calling `Display::fmt` directly on a `Chart`
outputs the frame/axes but leaves the Braille canvas blank.

### Completion criteria

- `plot(1:10, (1:10).^2)` renders a parabola in the terminal.
- `title("T"); xlabel("x"); ylabel("y"); plot(x, y)` shows annotations.
- `plot(x, y)` without the `plot` feature returns an actionable error.

---

## Phase 29b — SVG/PNG file export (v0.36.0) ✅

### What's new

- **`plot(x, y, 'file.svg')`** — saves a connected line chart as an SVG vector
  graphic (800 × 600). Requires the `plot-svg` feature.
- **`plot(x, y, 'file.png')`** — same but produces a 800 × 600 PNG raster image.
- **`scatter(x, y, 'file.svg')` / `scatter(x, y, 'file.png')`** — scatter
  (point cloud) file variants.
- **1-arg inferred-x form** — `plot(y, 'f.svg')` infers x = 1:numel(y),
  matching the terminal behaviour.
- **Annotations in file output** — `title`, `xlabel`, `ylabel` are embedded in
  the SVG/PNG and cleared after each render call.
- **Auto-range** — x and y extents are computed from data with a 5 % margin;
  single-point inputs use ± 1 padding.
- **`plotters 0.3` additional features required** — `line_series` (for
  `LineSeries`) and `ttf` (for TrueType text rendering in the bitmap backend).

### Dispatch rules

```
plot(v)            →  ASCII to terminal      requires plot
plot(x, y)         →  ASCII to terminal      requires plot
plot(x, y, 'ascii') →  ASCII to terminal      requires plot
plot(x, y, 'f.svg') →  SVG file export        requires plot-svg
plot(x, y, 'f.png') →  PNG file export        requires plot-svg
```

`scatter` follows identical dispatch rules.

### Implementation

All four file paths (line/scatter × SVG/PNG) share a single `render_file`
function in `crates/ccalc-plot/src/file.rs` (compiled under
`#[cfg(feature = "plot-svg")]`). The backend differs (SVGBackend vs
BitMapBackend); the chart-building logic is shared.

### Completion criteria

- `plot(x, sin(x), 'wave.svg')` creates a valid SVG file containing `<svg`.
- `plot(x, sin(x), 'wave.png')` creates a file whose first 8 bytes match the
  PNG magic number `\x89PNG\r\n\x1a\n`.
- `scatter` produces equivalent files.
- `title`/`xlabel`/`ylabel` text appears verbatim in the SVG output.
- `FigureState` is cleared after each file render (second render does not
  inherit annotations from the first).
- 9 integration tests in `crates/ccalc-plot/tests/svg_png_tests.rs` (gated
  `#[cfg(feature = "plot-svg")]`).

---

## Phase 29c — Bar, stem, log-scale, multi-series (planned)

`bar(v)`, `stem(x, v)`, `hist(v, n, 'f.svg')`, `stairs`, `loglog`,
`semilogx`, `semilogy`, multi-series `plot(x, Y)` with colour cycle.

## Phase 29d — 3-D plots (planned)

`plot3(x, y, z)` and `scatter3(x, y, z)` using orthographic projection
(az = −37.5°, el = 30°, matching MATLAB defaults). Infrastructure lives in
`crates/ccalc-plot/src/proj3d.rs` (no feature gate).

---

## See also

- [Plot functions guide](../guide/plot.md)
- [Plugins guide](../guide/plugins.md)
- [Phase 28 — Plugin Architecture](./phase28-plugins.md)
