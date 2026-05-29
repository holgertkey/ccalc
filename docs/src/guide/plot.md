# Plot Functions

ccalc supports terminal and file-based plotting via the `ccalc-plot` plugin crate.
Two rendering tiers are available:

| Feature flag | Backend | Enables |
|---|---|---|
| `plot` | `textplots` | ASCII Braille chart printed to terminal |
| `plot-svg` | `plotters` | SVG and PNG file export (default 800 × 600 px, customisable via `figure`) |
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

## 3D surface plots

### `meshgrid(x)` / `meshgrid(x, y)`

Generates coordinate matrices for evaluating functions on a 2D grid —
the standard prerequisite for `surf` and `mesh`.

| Call | Result |
|---|---|
| `[X, Y] = meshgrid(x, y)` | X is M×N (each row copies `x`); Y is M×N (each column copies `y`) |
| `[X, Y] = meshgrid(x)` | square N×N grid using `x` for both axes |
| `X = meshgrid(x, y)` | single-output form — returns only the X matrix |

```matlab
[X, Y] = meshgrid(-2:0.1:2, -2:0.1:2);
Z = exp(-(X.^2 + Y.^2));   % Gaussian bell
```

### `surf(X, Y, Z)` / `surf(X, Y, Z, 'file.svg')`

Colored 3D surface plot. X, Y, Z must all have the same dimensions (M×N
from `meshgrid`).

**ASCII tier** (`--features plot`): projects each column's maximum Z as a
vertical bar — an elevation silhouette. Prints `title`, `xlabel`, `ylabel`,
`zlabel` as header/footer. `colormap` is ignored.

**File tier** (`--features plot-svg`): renders the surface as a colored grid
of row and column `LineSeries`, each segment colored by local Z value through
the active colormap. Chart axes: X horizontal, Z (our height) vertical, Y depth.

```matlab
[X, Y] = meshgrid(-3:0.2:3, -3:0.2:3);
Z = sin(sqrt(X.^2 + Y.^2));

title('Sine wave surface')
colormap('viridis')
surf(X, Y, Z)                          % ASCII preview

surf(X, Y, Z, 'surface.svg')          % SVG file
```

### `mesh(X, Y, Z)` / `mesh(X, Y, Z, 'file.png')`

Wireframe 3D surface. Same API as `surf`; in ASCII mode the output is
identical. In file mode only row lines are drawn (no column fill lines),
giving a sparser wireframe appearance.

```matlab
[X, Y] = meshgrid(-2:0.2:2, -2:0.2:2);
Z = X.^2 - Y.^2;            % saddle surface

colormap('jet')
mesh(X, Y, Z, 'saddle.svg')
```

Both functions accept the same annotations as other plot functions
(`title`, `xlabel`, `ylabel`, `zlabel`, `xlim`, `ylim`, `zlim`,
`colormap`).

---

## Contour plots

Render 2D isolines (contour lines) or filled contour regions for a scalar field
defined on a meshgrid.

### `contour(X, Y, Z)` / `contour(X, Y, Z, n)` / `contour(X, Y, Z, n, 'file')`

Draws `n` evenly-spaced contour isolines.

- `X`, `Y` — coordinate matrices from `meshgrid`.
- `Z` — scalar-field matrix, same size as `X` and `Y`.
- `n` — number of contour levels (default `10`).
  Levels are placed evenly inside the Z range (never at the exact min/max).
- Without a path: ASCII tier prints a character-art density map (dimensions
  from `$COLUMNS` × `$LINES`, default 80 × 24) where each character encodes
  the Z band of the corresponding sample point (palette: `" .:-=+*#"`).
- With a `.svg` or `.png` path: file tier draws each isoline as a colored
  `LineSeries`, with colors cycling through the active colormap.

### `contourf(X, Y, Z)` / `contourf(X, Y, Z, n)` / `contourf(X, Y, Z, n, 'file')`

Filled contour chart. Same API as `contour`.

- ASCII tier: identical to `contour` (character-art density map).
- File tier: colors each grid cell by its Z band using the active colormap,
  then draws the contour isolines on top.

**Algorithm:** marching squares (classic isoline extraction per 2×2 cell).
The saddle-point ambiguity is resolved with the simple split convention.

```matlab
[X, Y] = meshgrid(-2:0.05:2, -2:0.05:2);
Z = exp(-X .^ 2 - Y .^ 2);

% ASCII density map (10 levels)
contour(X, Y, Z)

% SVG with 8 levels
title('Gaussian bell')
xlabel('x')
ylabel('y')
contour(X, Y, Z, 8, 'gauss.svg')

% PNG filled contour
colormap('viridis')
contourf(X, Y, Z, 8, 'gauss_filled.png')

% Saddle function — shows both positive and negative regions
Z2 = X .* exp(-X .^ 2 - Y .^ 2);
colormap('hot')
contour(X, Y, Z2, 12, 'saddle.svg')
```

Both functions accept `title`, `xlabel`, `ylabel`, `xlim`, `ylim`, and
`colormap` annotations, which are consumed by the render call.

### `clabel()`

Enables **contour level labels** for the *next* `contour` or `contourf` call.
The flag is consumed (cleared) by the render, matching the single-shot semantics
of `grid`, `colorbar`, and similar state annotations.

**ASCII tier:** prints a `Levels: …` footer line after the chart listing all level
values formatted to 2 decimal places.

**File tier:** places a text label at the midpoint of the longest marching-squares
segment for each level. Label color matches the isoline color; font size scales with
`fontsize(n)` (default 10 pt, proportional to the axis-label size).

```matlab
[X, Y] = meshgrid(-2:0.05:2, -2:0.05:2);
Z = exp(-X .^ 2 - Y .^ 2);

% ASCII with level footer
clabel()
contour(X, Y, Z, 6)

% SVG with inline labels
title('Gaussian bell — labeled contours')
xlabel('x')
ylabel('y')
clabel()
contour(X, Y, Z, 8, 'gauss_labeled.svg')

% contourf also respects clabel()
colormap('viridis')
clabel()
contourf(X, Y, Z, 8, 'gauss_filled_labeled.svg')
```

---

## Multi-panel layout

`subplot`, `hold`, and `savefig` work together to compose figures with multiple panels
or overlaid series.

### `subplot(rows, cols, index)`

Activates panel `index` (1-based, row-major) in a `rows × cols` grid.
Once called, ccalc enters *accumulating mode*: all subsequent plot calls
(`plot`, `scatter`, `bar`, `stem`, `stairs`, `hist`, `fill`, `area`, `quiver`) are
buffered instead of rendered immediately. Annotations (`title`, `xlabel`, `ylabel`, `xlim`, `ylim`,
`legend`, `grid`, `text`) set after the render call are collected for the current panel
and consumed at commit time.

Calling `subplot` a second time commits the current panel and starts the next one.
`savefig` commits the last panel and writes the composed figure.

```matlab
x = linspace(0, 2*pi, 60);

subplot(2, 2, 1);
title('sin(x)');
plot(x, sin(x));

subplot(2, 2, 2);
title('cos(x)');
plot(x, cos(x));

subplot(2, 2, 3);
bar([3 1 4 1 5 9 2 6]);

subplot(2, 2, 4);
hist(randn(1, 200), 20);

savefig('out.svg');
```

### `hold('on')` / `hold('off')`

Overlay multiple series in a single chart panel.

- `hold('on')` — enables accumulating mode; subsequent plot calls push series
  into the current panel without rendering.
- `hold('off')` — disables accumulating mode and, if no `subplot` is active,
  immediately renders the accumulated series to the terminal (ASCII tier).
  For file output, call `savefig` before `hold('off')`.

```matlab
x = linspace(0, 2*pi, 80);

% ASCII overlay: both series rendered at hold('off')
hold('on');
plot(x, sin(x));
plot(x, cos(x));
hold('off');

% File overlay via subplot + savefig
subplot(1, 1, 1);
title('sin and cos overlay');
hold('on');
plot(x, sin(x));
plot(x, cos(x));
hold('off');
savefig('overlay.svg');
```

### `savefig('path')`

Commits the last pending panel and renders all accumulated panels to a single
SVG or PNG file (requires `--features plot-svg`). The grid layout is determined
by the `rows × cols` dimensions passed to the `subplot` calls.

When used without `subplot` (only with `hold`), the single panel fills the
entire canvas.

---

## False-colour images (imagesc)

Render a matrix as a heat-map — each cell is coloured according to its value.

### `imagesc(Z)` / `imagesc(Z, path)`

- `Z` — any numeric matrix.
- Without a path: ASCII tier prints a character-art grid using 10 density
  characters (`" .:-=+*#@█"`) mapped from `Z_min` to `Z_max`.
  Grid dimensions adapt to terminal width (`$COLUMNS`, default 80).
- With a `.svg` or `.png` path: file tier draws one filled `Rectangle` per
  cell, scaled to the canvas. Canvas size comes from `figure(w, h)` (default
  800 × 600 px). Requires `--features plot-svg`.

### `colormap('name')` / `colormap(M)`

Set the active colormap for the **next** `imagesc` call (consumed and cleared
together with other `FigureState` annotations). Case-insensitive.

**Named colormaps:**

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

**Custom colormap from matrix:**

Pass an N×3 matrix where each row is an RGB control point with values in [0, 1].
The colormap is linearly interpolated between control points.

```matlab
% Two-stop blue → red
colormap([0 0 1; 1 0 0])
imagesc(Z, 'heat.svg')

% Three-stop blue → yellow → red
colormap([0 0 1; 1 1 0; 1 0 0])
imagesc(Z, 'custom.svg')
```

### `colorbar()`

Appends a colour-scale legend strip to the right side of the exported image
(80 px wide, with 5 tick labels at 0 %, 25 %, 50 %, 75 %, 100 % of the data
range). Silently ignored in ASCII mode.

```matlab
% ASCII heat-map
Z = reshape(1:100, 10, 10);
imagesc(Z)

% SVG with viridis colormap and colorbar
colormap('viridis')
colorbar()
title('Signal strength')
imagesc(Z, 'heat.svg')

% Mandelbrot set — colormap changes false-colour appearance
N = 200; max_iter = 60;
x = linspace(-2.5, 1.0, N);
y = linspace(-1.2, 1.2, N);
Z = zeros(N, N);
for row = 1:N
  for col = 1:N
    c = x(col) + y(row)*1i;
    z = 0;
    for k = 1:max_iter
      if abs(z) > 2, break; end
      z = z^2 + c;
    end
    Z(row, col) = k;
  end
end
colormap('inferno')
colorbar()
title('Mandelbrot set')
imagesc(Z, 'mandelbrot.svg')
```

---

## Style strings and colors

### Color specification forms

Five ways to specify a color, accepted by all plot functions that support styling:

| Form | Example | Description |
|---|---|---|
| Single-letter code | `'r'`, `'b'` | MATLAB-compatible short codes |
| Full color name | `'red'`, `'orange'` | Full English names |
| Hex `#RRGGBB` | `'#FF4400'` | 24-bit hex color |
| 1×3 RGB matrix | `[1 0.27 0]` | Row vector with values in [0, 1] |
| `'color'`, value | `'color', 'red'` | Named argument (for bar/stem/hist/quiver) |

**Single-letter codes:**

| Code | Color | Code | Color |
|---|---|---|---|
| `r` | red | `c` | cyan |
| `g` | green | `m` | magenta |
| `b` | blue | `y` | yellow |
| `k` | black | `w` | white |

**Additional named colors** (full names only, not single-letter):
`orange`, `purple`, `gray` / `grey`

### Style strings for `plot`, `scatter`, `fill`, `area`

These functions accept an optional MATLAB-compatible *style string* before the file
path. The string combines a color code, a marker code, and/or a line-style code in
any order.

| Code | Meaning |
|---|---|
| `r` `g` `b` `c` `m` `y` `k` `w` | Single-letter color |
| full name or `#RRGGBB` | Full color name or hex (style string is the entire argument) |
| `.` `o` `x` `+` `*` `s` `d` `^` | Marker (file export only) |
| `-` | Solid line (default) |
| `--` | Dashed line |
| `-.` | Dash-dot line |
| `:` | Dotted line |

```matlab
x = linspace(0, 2*pi, 80);

% Single-letter code: red dashed line
plot(x, sin(x), 'r--')

% Full color name
plot(x, sin(x), 'orange')

% Hex color
plot(x, cos(x), '#1A6ECC')

% 1×3 RGB matrix (values in [0, 1])
plot(x, sin(x), [0.8 0.2 0.1])

% Blue scatter with dot markers
scatter(x, cos(x), 'b.')

% Green solid line to SVG
plot(x, sin(x), 'g-', 'wave.svg')

% Red fill
fill([0, 1, 0.5], [0, 0, 1], 'r', 'tri.svg')
```

### Color for `bar`, `stem`, `hist`, `quiver`

These functions do not use a trailing style string (to avoid ambiguity with
data arguments). Use the `'color'` named argument instead:

```matlab
% Color name
bar([1 3 2 5 4], 'color', 'red')

% Hex color
stem(x, sin(x), 'color', '#FF8800')

% Full name in hist
hist(randn(1, 500), 20, 'color', 'purple')

% Quiver with named color
[X, Y] = meshgrid(-2:2, -2:2);
quiver(X, Y, -Y, X, 'color', 'blue')

% RGB matrix form also works
bar([1 3 2 5 4], 'color', [0.2 0.6 1.0])
```

> **Note:** In ASCII (textplots) mode, color and line-style are ignored because
> the backend is monochrome Braille. Style specifications still parse without error.

---

## Filled polygons and areas

### `fill(x, y)` / `fill(x, y, style)` / `fill(x, y, style, 'file')`

Filled polygon. `x` and `y` are coordinate vectors of the polygon vertices; the
shape is automatically closed (last vertex connects back to the first).

**ASCII tier:** prints a bounding-box density block with a `░` fill character plus
an outline using `textplots`.

**File tier:** draws a plotters `Polygon` element filled at 40 % opacity, with the
full-opacity outline drawn as a `LineSeries` on top.

```matlab
% Filled triangle
fill([0, 1, 0.5], [0, 0, 1])

% Red-filled triangle → SVG
fill([0, 1, 0.5], [0, 0, 1], 'r', 'triangle.svg')
```

### `area(y)` / `area(x, y)` / `area(x, y, style)` / `area(x, y, style, 'file')`

Filled area under a curve. The curve is closed along `y = 0` to form a polygon
(equivalent to `fill` with an added baseline segment).

```matlab
x = linspace(0, 2*pi, 80);

% ASCII area preview
area(x, sin(x) + 1)

% Blue area under sine wave → SVG
area(x, sin(x) + 1, 'b', 'area_sine.svg')
```

---

## Drawing primitives

Phase 32a adds three low-level drawing functions that complement `fill` and `area`.
All three participate in `hold`/`subplot` accumulation and `savefig` exactly like
the other chart functions.

### `line(x, y)` / `line(x, y, style)` / `line(x, y, style, 'file')`

MATLAB-compatible alias for [`plot`](#plotxy--plotxy-style--plotxy-style-file).
Accepts the same arguments, style strings, and file-export path.

```matlab
x = linspace(0, 2*pi, 64);
line(x, sin(x), 'b-', 'sine.svg')
```

### `patch(x, y)` / `patch(x, y, color)` / `patch(x, y, color, 'file')`

MATLAB-compatible alias for [`fill`](#fillx-y--fillx-y-style--fillx-y-style-file).
Draws a filled polygon from vertex vectors `x` and `y`.

```matlab
% Cyan-filled triangle → SVG
patch([0, 1, 0.5], [0, 0, 1], 'c', 'triangle.svg')
```

### `rectangle(x, y, w, h)` / `rectangle([x y w h])` / `rectangle(..., color)` / `rectangle(..., color, 'file')`

Draws an axis-aligned filled rectangle defined by its origin `(x, y)`,
`width`, and `height`. The bounding box is converted to a 4-vertex polygon
`[x, x+w, x+w, x] × [y, y, y+h, y+h]` and rendered via `render_fill_xy`.

**Two input forms:**

| Form | Syntax |
|------|--------|
| 4-scalar | `rectangle(x, y, w, h)` |
| vector   | `rectangle([x y w h])` |

```matlab
% Green rectangle (4-scalar form) → SVG
rectangle(0.1, 0.2, 0.6, 0.4, 'g', 'rect.svg')

% Magenta rectangle (vector form) → SVG
rectangle([0.1, 0.2, 0.6, 0.4], 'm', 'rect_vec.svg')

% Combined: sine curve inside a bounding box
hold('on')
line(x, sin(x), 'b-')
rectangle(0, -1, 2*pi, 2, 'k--')
title('sine + bounding box')
savefig('sine_box.svg')
```

See also: `examples/primitives_demo/primitives_demo.calc`

---

## Statistical extensions

Phase 32b adds two statistical chart functions.

### `errorbar(x, y, e)` / `errorbar(x, y, e, style)` / `errorbar(x, y, e, style, 'file')`

Draws a line plot with **symmetric** error bars: each point `(x[i], y[i])` gets
a vertical cap spanning `[y[i] - e[i], y[i] + e[i]]`.

### `errorbar(x, y, e_low, e_high)` / `errorbar(x, y, e_low, e_high, style)` / `errorbar(x, y, e_low, e_high, style, 'file')`

**Asymmetric** form: the lower extent is `y[i] - e_low[i]` and the upper extent
is `y[i] + e_high[i]`, allowing different uncertainties in each direction.

All arrays must have the same length. The optional style argument accepts the
same color/line-style strings as `plot`. Without a file path the result is
printed as a compact ASCII table with `±` notation.

**ASCII tier:** compact table `x | y ± e` (or `x | y + e_high - e_low`).

**File tier:** three `PathElement` segments per point (vertical shaft, lower cap,
upper cap) plus a `Circle` centre dot. Cap width = 3 % of the x-axis range.

```matlab
x = 1:5;
y = [2.1, 3.4, 2.8, 4.2, 3.7];

% Symmetric — same error on each side
e = [0.3, 0.2, 0.4, 0.25, 0.35];
xlabel('Sample')
ylabel('Value')
title('Symmetric error bars')
errorbar(x, y, e, 'b', 'errorbar_sym.svg')

% Asymmetric
e_low  = [0.1, 0.3, 0.2, 0.15, 0.4];
e_high = [0.4, 0.1, 0.5, 0.3,  0.2];
errorbar(x, y, e_low, e_high, 'r', 'errorbar_asym.svg')

% Overlay errorbar on a line plot (hold mode)
hold('on')
plot(1:5, 0.8*(1:5) + 0.5, 'k--')
errorbar(x, y, e, 'b')
title('Line + error bars')
savefig('errorbar_with_line.svg')
```

### `scatter(x, y, sz, c)` — per-point color form

When `scatter` receives **four numeric arguments** `(x, y, sz, c)`, each point
is colored individually by mapping the scalar `c[i]` through the active colormap
(default: `viridis`).

- `sz` — marker radius in pixels. Either a scalar (broadcast to all points) or a
  vector of the same length as `x`.
- `c` — scalar color values; automatically normalized to `[min(c), max(c)]` before
  the colormap lookup.
- Change the colormap with `colormap(name)` before the `scatter` call.

**ASCII tier:** degrades gracefully to a monochrome `textplots` scatter chart.

**File tier:** each point is a `Circle` element whose fill color comes from
`apply_colormap_spec(c_normalized)`.

```matlab
n  = 20;
x  = linspace(0, 2*pi, n);
y  = sin(x);
c  = cos(x);          % values drive the colormap

% Uniform size, viridis (default)
scatter(x, y, 6, c, 'scatter_viridis.svg')

% Per-point size, jet colormap
colormap('jet')
sz = 3 + 7 * (c - min(c)) / (max(c) - min(c));
scatter(x, y, sz, c, 'scatter_jet.svg')

% Two ColorScatter series in hold mode
x2 = linspace(0, 2*pi, n);
y2 = cos(x2);
hold('on')
scatter(x,  y,  5, c)
scatter(x2, y2, 5, sin(x2))
title('Two ColorScatter series')
savefig('scatter_hold.svg')
```

See also: `examples/errorbar_demo/errorbar_demo.calc`,
`examples/scatter_color_demo/scatter_color_demo.calc`

---

## Pie charts

Phase 32c adds pie chart support through the `pie` function.

### `pie(v)` / `pie(v, labels)` / `pie(v, explode)` / `pie(v, explode, labels)` / `pie(v, ..., 'file')`

Renders a proportional pie chart from the numeric vector `v`.  Each slice covers
an angular fraction equal to `v[i] / sum(v)`.  Values must be non-negative and
their sum must be positive.

**Argument type detection** (flexible ordering):
- `Cell` array of strings → slice labels.
- Numeric vector (same length as `v`) → per-slice explode offsets (see below).
- String ending in `.svg`/`.png` → output file path (requires `plot-svg`).

**Explode:** when `explode[i] > 0`, slice `i` is shifted radially outward by
`explode[i] × 0.08 × r` from the chart center.

**ASCII tier:** horizontal bar-art table with a 20-character bar per slice.
Four rotating fill characters (`█ ▓ ▒ ░`) visually distinguish slices; empty
bar space is filled with `·`; a `:` marker appears at the midpoint (position 10)
of every bar for scale reference; exploded slices get a `◄` suffix after the label.

```
pie chart:
  Work      ████████··········:··········  30.0% ◄
  Sleep     █████·············:··········  20.0%
  Exercise  ████··············:··········  15.0%
  Leisure   ██████████········:··········  25.0%
  Eating    ██················:··········  10.0%
```

**File tier:** one `Polygon` wedge per slice built from 64 arc points plus the
center point (65 vertices total).  The chart is drawn in a `(-1..1) × (-1..1)`
Cartesian space with axes and mesh hidden.  Labels are placed at radius
`r × 1.18` using `Text` elements.  Slices cycle through the 7-color Octave
palette; set `colormap('name')` before calling `pie` to use a different palette.

```matlab
v = [30, 20, 15, 25, 10];
labels = {'Work', 'Sleep', 'Exercise', 'Leisure', 'Eating'};

% ASCII output
pie(v)
pie(v, labels)

% Explode first slice outward
explode = [0.1, 0, 0, 0, 0];
pie(v, explode, labels)

% File export
pie(v, 'pie_basic.svg')
pie(v, labels, 'pie_labels.svg')
pie(v, explode, labels, 'pie_explode.svg')
```

See also: `examples/pie_demo/pie_demo.calc`

---

## Dual Y axis

Phase 32d adds dual Y-axis support through `yyaxis`.

### `yyaxis('left')` / `yyaxis('right')`

Switches the **active Y axis**.  All subsequent `plot`, `scatter`, `ylabel`, and
`ylim` calls are routed to that axis until the axis is switched again.

Both calls implicitly enable hold mode so that series from both sides accumulate
before rendering.  The chart is flushed automatically when:

- `yyaxis('left')` is called again while right-axis series are pending
  (i.e. at the start of the next dual-axis block), or
- `savefig('path.svg')` commits all pending panels to a file.

Call `hold('off')` to render the chart to the terminal immediately without
starting a new block.

**ASCII rendering** draws both curves on a single character grid; left-axis
series use `.` and right-axis series use `*`.  The footer lines show the actual
Y range for each axis:

```
Temperature and Humidity
+------------------------------------------------------------------------+
|                                          *****                         |
|                                      .***     ********                 |
|                                  ..***                *****            |
|                              ...***                        ***         |
+------------------------------------------------------------------------+
x: Time (h)
y (left)  . : Temperature (C)  [18 .. 23]
y (right) * : Humidity (%)     [60 .. 70]
```

**SVG / PNG rendering** uses the plotters `DualCoordChartContext` so the left
and right Y axes each carry independent tick labels and optional grid lines.

```matlab
t       = [0, 1, 2, 3, 4, 5];
temp_C  = [18, 19, 21, 23, 22, 20];
humid_p = [60, 62, 65, 70, 68, 64];

% ASCII output — renders automatically when the next yyaxis block begins
yyaxis('left');
ylabel('Temperature (C)');
plot(t, temp_C, 'b-');

yyaxis('right');
ylabel('Humidity (%)');
plot(t, humid_p, 'r--');

xlabel('Time (h)');
title('Temperature and Humidity');

% SVG output
yyaxis('left');           % <-- also flushes the ASCII chart above
ylabel('Temperature (C)');
plot(t, temp_C, 'b-');

yyaxis('right');
ylabel('Humidity (%)');
plot(t, humid_p, 'r--');

xlabel('Time (h)');
title('Temperature and Humidity');
savefig('examples/yyaxis_demo/output/yyaxis_basic.svg');
```

See also: `examples/yyaxis_demo/yyaxis_demo.calc`

---

## Polar plots

### `polar(theta, r)` / `polar(theta, r, style)` / `polar(theta, r, 'file')`

Converts polar coordinates `(r, theta)` to Cartesian `(x, y)` using
`x = r·cos(θ)`, `y = r·sin(θ)` and renders a connected line plot.

`theta` is in **radians**.

```matlab
theta = linspace(0, 2*pi, 200);

% Unit circle
polar(theta, ones(size(theta)))

% Rose curve: r = |cos(2θ)|
polar(theta, abs(cos(2*theta)), 'rose.svg')

% Archimedean spiral: r = θ/(2π)
polar(theta, theta / (2*pi), 'spiral.svg')
```

---

## Vector field plots

### `quiver(x, y, u, v)` / `quiver(x, y, u, v, 'file')`

Draws a vector field: at each point `(x[i], y[i])` an arrow is drawn in the
direction `(u[i], v[i])`.

- All four arrays must have the same length (or the same total element count when
  meshgrid matrices are passed — they are flattened in row-major order).
- Arrow scale: the longest arrow is normalised to 80 % of the minimum grid spacing,
  so arrows never overlap adjacent grid cells.

**ASCII tier:** places a Unicode directional arrow character (`→ ↗ ↑ ↖ ← ↙ ↓ ↘`)
at the grid position of each origin point.

**File tier:** each arrow is drawn as a shaft (`PathElement`) plus a filled
triangular arrowhead at the tip.

```matlab
% Simple rotational flow: u = -y, v = x
[X, Y] = meshgrid(-2:1:2, -2:1:2);
U = -Y;
V = X;

% ASCII render
title('Rotational flow')
quiver(X, Y, U, V)

% SVG export
quiver(X, Y, U, V, 'flow.svg')
```

---

## Text annotations

### `text(x, y, 'str')` / `text(x, y, 'str', 'file')`

Places a text label at the data coordinates `(x, y)`.

Text annotations are stored in `FigureState.annotations` and are flushed
alongside plot data at the next render call or at `savefig` / `hold('off')`.
They do **not** trigger an immediate render on their own.

**ASCII tier:** annotations are printed below the chart as
`(x, y): label` lines.

**File tier:** annotations are drawn as `Text` elements at their data
coordinates using a 12-pt sans-serif font.

```matlab
% Annotate a quiver plot
text(0.0, 0.0, 'origin')
text(2.0, 2.0, 'tip region')
quiver(x, y, u, v, 'annotated.svg')

% Annotate any plot
x = linspace(0, 2*pi, 80);
text(pi/2, 1.0, 'peak')
text(3*pi/2, -1.0, 'trough')
plot(x, sin(x), 'sine.svg')
```

---

## Canvas size

### File export: `figure(width, height)`

Sets the output canvas size in pixels for the **next** SVG or PNG export.
Applies to all file-export functions: `plot`, `scatter`, `bar`, `hist`, `fill`,
`area`, `polar`, `quiver`, `surf`, `mesh`, `contour`, `contourf`, and `savefig`.

- Width and height must be integers in the range **1–16384**.
- The size persists across panels (like `colormap`) and is cleared when the
  figure state resets after a render.
- Has no effect in ASCII (terminal) mode — ASCII chart dimensions follow the
  terminal size instead (see below).

```matlab
% Wide landscape chart
figure(1200, 400)
plot(x, sin(x), 'wide.svg')

% Square heatmap
figure(600, 600)
colormap('viridis')
imagesc(Z, 'square.svg')

% Multi-panel at HD resolution
figure(1920, 1080)
subplot(2, 2, 1); plot(x, sin(x)); title('sin')
subplot(2, 2, 2); plot(x, cos(x)); title('cos')
subplot(2, 2, 3); bar([1 2 3 4]);
subplot(2, 2, 4); hist(randn(1, 200), 20);
savefig('hd_grid.png')
```

### ASCII output: terminal auto-detection

ASCII charts automatically adapt to the terminal size by reading the standard
environment variables `$COLUMNS` (width, default 80) and `$LINES` (height,
default 24) at render time.

| Chart type | Uses `$COLUMNS` | Uses `$LINES` |
|---|---|---|
| `plot`, `scatter`, `bar`, `stem`, `stairs` | Yes (Braille canvas width) | Yes (Braille canvas height) |
| `fill`, `area` | Yes (character grid) | Yes |
| `hist` | Yes (bar width) | — |
| `contour`, `contourf` | Yes | Yes |
| `surf`, `mesh` | — | Yes (elevation height) |
| `quiver` | Yes | Yes |

Set these variables in your shell before running ccalc to get larger charts:

```bash
export COLUMNS=120
export LINES=40
ccalc
```

Or inline for a single script:

```bash
COLUMNS=120 LINES=40 ccalc -q myscript.calc
```


---

## Figure appearance

The following functions adjust the visual appearance of the next rendered figure.
Like other annotations, they are stored in `FigureState` and consumed by the next render call.
These settings apply to **SVG/PNG file output only**; ASCII charts are monochrome and their
geometry is fixed by the terminal size.

### Theme and background color

| Function | Effect |
|---|---|
| `theme('light')` | Light theme: white background, black text and axes (default) |
| `theme('dark')` | Dark theme: Catppuccin Mocha palette (`#1E1E2E` bg, `#CDD6F4` text) |
| `bgcolor(color)` | Override the figure background color only (beats the theme) |

`bgcolor` accepts any [color specification](#color-specification): a color name string,
a hex code `'#RRGGBB'`, or a 1×3 RGB matrix with values in `[0, 1]`.

```matlab
theme('dark')
plot(x, sin(x), 'sin_dark.svg')

bgcolor('#F5F5F5')     % light grey background, keeps other defaults
plot(x, cos(x), 'cos_grey.svg')
```

### Font and stroke sizes

| Function | Effect |
|---|---|
| `fontsize(n)` | Override title and axis-label font size (pixels) |
| `linewidth(f)` | Override default line stroke width for all series (pixels) |
| `markersize(n)` | Override default marker radius for all series (pixels) |

Per-series overrides are applied via named arguments appended to a single plot call:

```matlab
plot(x, y, 'r--', 'linewidth', 2)         % thick red dashed line
scatter(x, y, 'markersize', 5)             % larger dot markers
plot(x, y, 'linewidth', 1.5, 'markersize', 4)
```

Figure-level overrides apply to all series unless a per-series value is present:

```matlab
fontsize(14)
linewidth(2)
title('Thick lines')
plot(x, sin(x), 'sin_thick.svg')
```

### Grid color and width

| Function | Effect |
|---|---|
| `gridcolor(color)` | Override both bold and light grid line color |
| `gridwidth(n)` | Override grid line stroke width (pixels) |

Requires `grid('on')` to have any visible effect.

```matlab
grid('on')
gridcolor('#4080FF')
gridwidth(0.5)
plot(x, sin(x), 'blue_grid.svg')
```

### Axis mode

| Call | Effect |
|---|---|
| `axis('equal')` | Equal scaling — same data-units per pixel on both axes |
| `axis('tight')` | No margin — data range fills the chart area exactly |
| `axis('off')` | Hide all axis decorations (lines, ticks, labels) |
| `axis('on')` | Restore default axis display (cancels a previous `axis` call) |

```matlab
t = linspace(0, 2*pi, 120);
axis('equal')
plot(cos(t), sin(t), 'circle.svg')    % unit circle appears as a circle

axis('tight')
bar([3 1 4 1 5], 'tight_bar.svg')    % bars fill the chart with no margin

axis('off')
imagesc(Z, 'clean.svg')              % image only, no axis decorations
```

`axis('equal')` expands the tighter axis so data-units-per-pixel are equal on both axes.
`axis('tight')` removes the default 5 % margin around the data range.
Both apply to SVG/PNG output; ASCII charts are unaffected.

---

## File export

Append a file path as the **last** string argument (after the optional style string):

| Extension | Format | Notes |
|---|---|---|
| `'.svg'` | SVG vector graphic | Opens in any browser |
| `'.png'` | PNG raster | Default 800 × 600 px; override with `figure(w, h)` |
| `'ascii'` | Terminal chart | Forces ASCII even with `plot-svg` active |

`imagesc` always writes to a file (never prints a file path to the terminal).
The `colormap` and `colorbar` annotations apply only to `imagesc`.

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
| `colormap('name')` | Set colormap for next `imagesc` / `surf` / `mesh` / `contour` | Yes |
| `colorbar()` | Append colour-scale strip (file export only, `imagesc`) | Yes |
| `clabel()` | Enable level labels on the next `contour` / `contourf` render | Yes |
| `figure(w, h)` | Set SVG/PNG canvas size in pixels (1–16384); ASCII ignores it | Yes |
| `text(x, y, 's')` | Add label at data coordinate — flushed with next render | Yes |
| `theme('light'\|'dark')` | Set colour theme (SVG/PNG only) | Yes |
| `bgcolor(color)` | Override figure background color (beats theme) | Yes |
| `fontsize(n)` | Override title and axis-label font size in pixels | Yes |
| `linewidth(f)` | Override default line stroke width for all series | Yes |
| `markersize(n)` | Override default marker radius for all series | Yes |
| `gridcolor(color)` | Override grid line color (requires `grid('on')`) | Yes |
| `gridwidth(n)` | Override grid line stroke width (requires `grid('on')`) | Yes |
| `axis('equal'\|'tight'\|'off'\|'on')` | Axis scale mode / visibility (SVG/PNG only) | Yes |

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

- **Size (file export):** 800 × 600 px by default; override with `figure(width, height)` (1–16384 px).
- **Size (ASCII):** adapts to terminal `$COLUMNS` × `$LINES` (defaults 80 × 24).
- **Colours (multi-series):** 7-colour Octave palette — blue, orange, yellow, purple,
  green, cyan, dark red — cycling as needed.
- **Line plots:** `LineSeries` (1 px, series colour).
- **Scatter plots:** filled circles, 3 px radius.
- **Per-point color scatter (`scatter(x,y,sz,c)`):** `Circle` elements; each fill color mapped through the active colormap; radius from `sz` (scalar or per-point vector).
- **Contour labels (`clabel()` before `contour`/`contourf`):** one `Text` element per level, placed at the midpoint of the longest segment; color matches the isoline; font size scales with `fontsize(n)`.
- **Error bars (`errorbar`):** three `PathElement` segments (shaft + two caps) plus a `Circle` centre dot per data point; cap width = 3 % of x-range.
- **Pie charts (`pie`):** one `Polygon` wedge per slice (64 arc points + center); axes and mesh disabled; labels via `Text` at radius × 1.18; explode offsets along slice bisector.
- **Bar charts:** edge-to-edge `Rectangle` series; negative bars extend below baseline.
- **Stem plots:** `PathElement` vertical lines + `Circle` tip markers (4 px).
- **Histograms:** edge-to-edge `Rectangle` bins (blue fill).
- **3D line plots (`plot3`):** `LineSeries` over `(f64, f64, f64)` tuples via `plotters`
  3D Cartesian chart (`build_cartesian_3d`).
- **3D scatter plots (`scatter3`):** `Circle` elements at each 3D coordinate.
- **3D surface plots (`surf`):** colored row + column `LineSeries` grid on a 3D
  Cartesian chart; each line colored by local Z mean through the active colormap.
- **3D wireframe plots (`mesh`):** row-only `LineSeries` grid (sparser than `surf`).
- **False-colour images (`imagesc`):** one `Rectangle` per matrix cell, RGB colour
  from the active colormap LUT; optional 80 px colorbar strip on the right.
- **Axis range:** auto-computed from data with 5 % margin by default.
  `axis('tight')` removes the margin; `axis('equal')` enforces equal data-units/pixel;
  `axis('off')` hides all axis decorations. Single-point data uses ± 1.
- **Legend:** shown when `legend(...)` is set; drawn in the upper-right corner with
  a black border.

---

## Examples

- `examples/plot_file/plot_demo.calc` — ASCII `plot`/`scatter`, annotations
- `examples/plot_file/plot_file.calc` — `plot`/`scatter` to SVG/PNG
- `examples/plot_extended_file/plot_extended.calc` — `bar`, `stem`, `stairs`, `hist`,
  `loglog`/`semilogx`/`semilogy`, multi-series, `xlim`/`ylim`/`grid` (ASCII)
- `examples/plot_extended_file/plot_extended_file.calc` — same chart
  types exported to SVG/PNG, multi-series with `legend`+`grid`, histogram variants
- `examples/plot3_file/plot3_demo.calc` — `plot3`/`scatter3` ASCII 3D plots
- `examples/plot3_file/plot3_file.calc` — `plot3`/`scatter3` exported to SVG/PNG
- `examples/colormap/imagesc_demo.calc` — `imagesc` with all 8 colormaps + colorbar
- `examples/colormap/mandelbrot.calc` — Mandelbrot set rendered with `colormap('inferno')`
- `examples/colormap/julia.calc` — Julia set rendered with `colormap('magma')`
- `examples/surf_demo/surf_demo.calc` — sine wave surface + Gaussian bell (`surf`)
- `examples/surf_demo/mesh_demo.calc` — sine wave wireframe + saddle surface (`mesh`)
- `examples/contour_demo/contour_demo.calc` — `contour`, `contourf`, and `clabel()` level labels on Gaussian bell + saddle
- `examples/subplot_demo/subplot_demo.calc` — 2×2 grid: sin, cos, bar, hist (SVG export)
- `examples/hold_demo/hold_demo.calc` — overlaid sin and cos series using `hold on/off`
- `examples/fill_area_polar_demo/fill_area_polar_demo.calc` — `fill`, `area`, `polar`, style strings
- `examples/quiver_demo/quiver_demo.calc` — vector field with Unicode arrow grid
- `examples/color_system_demo/color_system_demo.calc` — Phase 30.5 unified color system: custom colormaps, full names, hex, RGB matrix, `'color'` named arg for bar/stem/hist/quiver
- `examples/figure_appearance_demo/figure_appearance_demo.calc` — Phase 30.6 figure appearance: `theme`, `bgcolor`, `fontsize`, `linewidth`, `markersize`, `gridcolor`, `gridwidth`, `axis`
- `examples/primitives_demo/primitives_demo.calc` — Phase 32a: `line`, `patch`, `rectangle` in hold mode and standalone
- `examples/errorbar_demo/errorbar_demo.calc` — Phase 32b: symmetric and asymmetric `errorbar`, hold-mode overlay with `plot`
- `examples/scatter_color_demo/scatter_color_demo.calc` — Phase 32b: per-point color `scatter(x,y,sz,c)` with viridis/jet colormaps and hold mode
- `examples/pie_demo/pie_demo.calc` — Phase 32c: `pie` chart with labels, explode, and file export
- `examples/yyaxis_demo/yyaxis_demo.calc` — Phase 32d: dual Y-axis — temperature vs humidity (ASCII + SVG), population vs growth rate (SVG)
- `examples/contour_demo/contour_demo.calc` already covers Phase 32e (`clabel()` calls included)

---

## See also

- [Plugins](./plugins.md) — how the `ccalc-plot` plugin is registered
- Run `help plot` in the REPL for a compact quick reference
