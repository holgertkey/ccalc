# Plot Functions

ccalc supports terminal and file-based plotting via the `ccalc-plot` plugin crate.
Two rendering tiers are available:

| Feature flag | Backend | Enables |
|---|---|---|
| `plot` | `textplots` | ASCII Braille chart printed to terminal |
| `plot-svg` | `plotters` | SVG and PNG file export (800 × 600 px) |
| `plot-all` | both | terminal + file export |

Build with the desired tier:

```bash
cargo build --release --features plot          # ASCII only
cargo build --release --features plot-svg      # file export only
cargo build --release --features plot-all      # both
```

Without a feature flag, calling a render function returns a helpful error suggesting
the rebuild command. Annotation functions (`title`, `xlabel`, `ylabel`, `xlim`, `ylim`,
`legend`, `grid`) always succeed in every build configuration.

---

## Chart types

All chart functions accept an optional trailing file path. When the last string argument
ends in `.svg` or `.png` the chart is saved to that file (requires `plot-svg`).
Without a file path the chart is rendered to the terminal (requires `plot`).

### `plot(y)` / `plot(x, y)` / `plot(x, M)`

Connected line chart.

- `y` — row or column vector; `x` inferred as `1:numel(y)` when omitted.
- `M` — M×N matrix: each **row** is drawn as a separate series. In SVG/PNG mode
  each series gets a distinct colour from the 7-colour Octave palette; `legend`
  labels the series.

```matlab
x = linspace(0, 2*pi, 80);
plot(x, sin(x))

% multi-series
M = [sin(x); cos(x); 0.5*sin(2*x)];
legend('sin', 'cos', '0.5 sin(2x)')
plot(x, M)
```

### `scatter(y)` / `scatter(x, y)`

Individual point cloud — use when connecting data points would imply false continuity.

```matlab
t = linspace(-2, 2, 50);
scatter(t, t.^2 + 0.3*randn(size(t)))
```

### `bar(y)` / `bar(x, y)`

Vertical bar chart. Bars extend from `y = 0`; negative values drop below the baseline.
Bar width is 40 % of the minimum x-spacing.

```matlab
months = 1:12;
rain   = [42 38 55 61 72 80 95 90 73 58 44 40];
xlabel('month')
ylabel('mm')
bar(months, rain)
```

### `stem(y)` / `stem(x, y)`

Discrete-sequence plot: a vertical line from `y = 0` to each tip, plus a circle marker.
Typical use: impulse/frequency responses and sampled signals.

```matlab
n = 0:15;
stem(n, 0.8 .^ n)
```

### `stairs(y)` / `stairs(x, y)`

Piecewise-constant (step-function) chart — each value is held until the next sample.
Useful for zero-order-hold signals, quantised waveforms, and control outputs.

```matlab
t = 0:0.5:4.5;
v = [0 0 1 1 2 2 1 1 0 0];
stairs(t, v)
```

### `hist(v)` / `hist(v, n)` / `hist(v, edges)`

Histogram. ASCII output (character bars) requires no feature flag; SVG/PNG requires `plot-svg`.

| Call | Bin specification |
|---|---|
| `hist(v)` | Sturges heuristic: `max(1, round(sqrt(numel(v))))` bins |
| `hist(v, n)` | Exactly `n` uniform bins |
| `hist(v, edges)` | Caller-supplied edge vector (length k+1 defines k bins) |

```matlab
data = randn(1, 200);
hist(data)          % auto bins
hist(data, 20)      % 20 uniform bins
hist(data, -3:0.5:3)   % explicit edges
```

### `loglog(x, y)` / `semilogx(x, y)` / `semilogy(x, y)`

Log-scale plots. Data is transformed with log₁₀ before rendering; non-positive values
are silently excluded. Axis labels are annotated with `[log₁₀]`.

| Function | X axis | Y axis |
|---|---|---|
| `loglog` | log₁₀ | log₁₀ |
| `semilogx` | log₁₀ | linear |
| `semilogy` | linear | log₁₀ |

```matlab
f = 10 .^ linspace(1, 5, 80);   % 10 Hz – 100 kHz
G = 1e6 * f .^ (-2);
loglog(f, G)
```

---

## 3D plots

### `plot3(x, y, z)` / `scatter3(x, y, z)`

Three-dimensional line and point cloud plots. All three vectors must have the same length.

**ASCII tier** (`--features plot`): projects `(x, y, z)` onto a 2D plane using an
orthographic projection with MATLAB-compatible default view angles
(azimuth = −37.5°, elevation = 30°). The projected points are rendered with `textplots`.
`xlabel` / `ylabel` / `zlabel` appear as labeled footer lines below the chart.

**File tier** (`--features plot-svg`): uses the `plotters` 3D Cartesian chart engine
(`build_cartesian_3d`). `plot3` draws a connected `LineSeries`; `scatter3` draws
filled circles at each point.

```matlab
% 3D helix — ASCII
t  = linspace(0, 4*pi, 120);
title('3D helix')
xlabel('x = cos(t)')
ylabel('y = sin(t)')
zlabel('z = t/(4π)')
plot3(cos(t), sin(t), t/(4*pi))

% Lissajous 3D — save to SVG
t2 = linspace(0, 2*pi, 200);
title('Lissajous 3D')
plot3(sin(3*t2), sin(2*t2), cos(t2), 'lissajous.svg')

% 3D scatter
scatter3(randn(1,80), randn(1,80), randn(1,80), 'cloud.svg')
```

---

## File export

Append a file path as the **last** string argument:

| Extension | Format | Notes |
|---|---|---|
| `'.svg'` | SVG vector graphic | Opens in any browser |
| `'.png'` | PNG raster, 800 × 600 px | |
| `'ascii'` | Terminal chart | Forces ASCII even with `plot-svg` active |

```matlab
x = linspace(0, 2*pi, 200);
title('sin(x)')
xlabel('x (radians)')
ylabel('amplitude')
plot(x, sin(x), 'wave.svg')

hist(randn(1, 500), 'dist.png')
```

---

## Annotation functions

Set annotations **before** the render call. All annotations are stored in a thread-local
`FigureState` and consumed (cleared) by the next render call.

```matlab
title('My Chart')
xlabel('time (s)')
ylabel('amplitude')
xlim([0, 10])
ylim([-1.2, 1.2])
grid('on')
plot(t, y)       % all annotations applied here, then cleared
```

| Function | Effect | Works without feature |
|---|---|---|
| `title('text')` | Chart title | Yes |
| `xlabel('text')` | X-axis label | Yes |
| `ylabel('text')` | Y-axis label | Yes |
| `zlabel('text')` | Z-axis label (consumed by `plot3`/`scatter3`) | Yes |
| `xlim([lo, hi])` | Override x-axis range | Yes |
| `ylim([lo, hi])` | Override y-axis range | Yes |
| `zlim([lo, hi])` | Override z-axis range (3D file export) | Yes |
| `legend(s1, s2, …)` | Series labels — applied in SVG/PNG multi-series charts | Yes |
| `grid` | Toggle grid on/off | Yes |
| `grid('on')` | Enable grid | Yes |
| `grid('off')` | Disable grid | Yes |

Grid defaults to **off**. The grid is visible in SVG/PNG output only; ASCII charts
ignore it.

Annotations not consumed before a second render call are **not** carried over:

```matlab
title('First plot')
plot(x, y1, 'a.svg')    % title applied
plot(x, y2, 'b.svg')    % no title — state was cleared by first render
```

---

## SVG/PNG chart properties

- **Size:** 800 × 600 px (fixed).
- **Colours (multi-series):** 7-colour Octave palette — blue, orange, yellow, purple,
  green, cyan, dark red — cycling as needed.
- **Line plots:** `LineSeries` (1 px, series colour).
- **Scatter plots:** filled circles, 3 px radius.
- **Bar charts:** edge-to-edge `Rectangle` series; negative bars extend below baseline.
- **Stem plots:** `PathElement` vertical lines + `Circle` tip markers (4 px).
- **Histograms:** edge-to-edge `Rectangle` bins (blue fill).
- **3D line plots (`plot3`):** `LineSeries` over `(f64, f64, f64)` tuples via `plotters`
  3D Cartesian chart (`build_cartesian_3d`).
- **3D scatter plots (`scatter3`):** `Circle` elements at each 3D coordinate.
- **Axis range:** auto-computed from data with 5 % margin; single-point data uses ± 1.
- **Legend:** shown when `legend(...)` is set; drawn in the upper-right corner with
  a black border.

---

## Examples

- `examples/plot_demo.calc` — ASCII `plot`/`scatter`, annotations
- `examples/plot_file/plot_file.calc` — `plot`/`scatter` to SVG/PNG
- `examples/plot_extended.calc` — `bar`, `stem`, `stairs`, `hist`,
  `loglog`/`semilogx`/`semilogy`, multi-series, `xlim`/`ylim`/`grid` (ASCII)
- `examples/plot_extended_file/plot_extended_file.calc` — same chart
  types exported to SVG/PNG, multi-series with `legend`+`grid`, histogram variants
- `examples/plot3_demo.calc` — `plot3`/`scatter3` ASCII 3D plots
- `examples/plot3_file/plot3_file.calc` — `plot3`/`scatter3` exported to SVG/PNG

---

## See also

- [Plugins](./plugins.md) — how the `ccalc-plot` plugin is registered
- Run `help plot` in the REPL for a compact quick reference
