# Phase 29 — Plot Engine

**Version:** 0.35.0 (Phase 29a — ASCII rendering)

Fills `PlotPlugin` with real rendering logic, building on the plugin
infrastructure added in Phase 28.

## Phase 29a — ASCII terminal rendering (v0.35.0)

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
  (plotters/SVG+PNG, Phase 29b), `plot-all` (both).

### Rendering notes

`textplots 0.8` renders data using Braille characters (U+2800–U+28FF).
The canvas is populated only via the method-chain call
`.lineplot(&data).display()`. Calling `Display::fmt` directly on a `Chart`
outputs the frame/axes but leaves the Braille canvas blank.

### Completion criteria

- `plot(1:10, (1:10).^2)` renders a parabola in the terminal.
- `title("T"); xlabel("x"); ylabel("y"); plot(x, y)` shows annotations.
- `plot(x, y)` without the `plot` feature returns an actionable error.
- All existing tests pass (1089 tests at time of completion).

## Phase 29b — SVG/PNG file output (planned)

`plot(x, y, "out.svg")` / `plot(x, y, "out.png")` via `plotters 0.3`.
Requires the `plot-svg` feature.

## Phase 29c — Bar and stem charts (planned)

`bar(x, y)` and `stem(x, y)` for discrete data.

## Phase 29d — 3-D plots (planned)

`plot3(x, y, z)` and `scatter3(x, y, z)` using orthographic projection
(az = −37.5°, el = 30°, matching MATLAB defaults). Infrastructure lives in
`crates/ccalc-plot/src/proj3d.rs` (no feature gate).

## See also

- [Plot functions guide](../guide/plot.md)
- [Plugins guide](../guide/plugins.md)
- [Phase 28 — Plugin Architecture](./phase28-plugins.md)
