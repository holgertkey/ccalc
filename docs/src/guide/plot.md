# Plot Functions

ccalc supports terminal and file-based plotting via the `ccalc-plot` plugin crate.
Two feature tiers are available:

| Feature flag | Backend | Enables |
|---|---|---|
| `plot` | `textplots` | ASCII Braille chart printed to terminal |
| `plot-svg` | `plotters` | SVG and PNG file export |
| `plot-all` | both | terminal + file export |

Build with the desired tier:

```bash
cargo build --release --features plot          # ASCII only
cargo build --release --features plot-svg      # file export only
cargo build --release --features plot-all      # both
```

Without a feature, calling a render function returns an informative error
suggesting the rebuild command. Annotation functions (`xlabel`, `ylabel`,
`title`) work in every build configuration.

---

## Terminal rendering (requires `plot`)

### `plot(y)` / `plot(x, y)`

Renders a connected line chart to the terminal using Braille characters.

- `y` — row or column vector of values
- `x` — optional explicit x-axis vector (same length as `y`); inferred as
  `1:numel(y)` when omitted

```matlab
x = linspace(0, 2*pi, 80);
plot(x, sin(x))
```

### `scatter(y)` / `scatter(x, y)`

Same as `plot` but draws individual dots instead of connected lines — use
when connecting the data points would imply false continuity.

```matlab
t = linspace(-2, 2, 50);
scatter(t, t.^2 + 0.3*randn(size(t)))
```

---

## File export (requires `plot-svg`)

Append a file path as the last argument. The extension determines the format:

| Last argument | Format | Notes |
|---|---|---|
| `'name.svg'` | SVG vector graphic | Opens in any browser |
| `'name.png'` | PNG raster image | 800 × 600 px |
| `'ascii'` | Terminal chart | Forces ASCII even when `plot-svg` is active |

### `plot(y, 'file.svg')` / `plot(x, y, 'file.svg')`

Saves a connected line chart to an SVG or PNG file.

```matlab
x = linspace(0, 2*pi, 200);

title('sin(x)')
xlabel('x (radians)')
ylabel('amplitude')
plot(x, sin(x), 'wave.svg')
```

### `scatter(y, 'file.svg')` / `scatter(x, y, 'file.svg')`

Saves a scatter (point cloud) chart to file.

```matlab
scatter(temp, ohms, 'resist.png')
```

Both `plot` and `scatter` support the 1-arg inferred-x form for file export:

```matlab
decay = exp(-linspace(0, 4, 60));
plot(decay, 'decay.svg')    % x inferred as 1:60
```

---

## Annotation functions

Set annotations **before** the render call. Each annotation is stored in a
thread-local `FigureState` and consumed (reset) by the next render call:

```matlab
title('My Chart')    % set title
xlabel('time (s)')   % set x-axis label
ylabel('amplitude')  % set y-axis label
plot(t, y)           % annotations applied here, then cleared
```

| Function | Effect |
|---|---|
| `title('text')` | Chart title — printed above (ASCII) or embedded (SVG/PNG) |
| `xlabel('text')` | X-axis label |
| `ylabel('text')` | Y-axis label |

Annotations not consumed before a second render call are **not** carried over:

```matlab
title('First plot')
plot(x, y1, 'a.svg')    % title applied
plot(x, y2, 'b.svg')    % no title — state was cleared by first render
```

---

## Chart properties (SVG/PNG)

- **Size:** 800 × 600 px (fixed in Phase 29b; configurable range planned for 29c)
- **Axis range:** auto-computed from data with a 5 % margin; single-point data
  uses ± 1 padding
- **Line plots:** `LineSeries` (solid blue line)
- **Scatter plots:** filled circles (3 px radius)
- Both axes are drawn and labelled

---

## See also

- [Plugins](./plugins.md) — how the `ccalc-plot` plugin is registered
- [Phase 29 — Plot engine](../ccalc/phase29-plot.md) — implementation notes
- `examples/plot_demo.calc` — ASCII terminal rendering demo
- `examples/plot_file/plot_file.calc` — SVG/PNG file export demo
