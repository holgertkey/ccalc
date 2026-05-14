# Plot Functions

ccalc supports ASCII terminal plots via the `ccalc-plot` plugin crate.
Rendering requires the `plot` Cargo feature:

```bash
cargo build --features plot
cargo run  --features plot
```

## Functions

### `plot(x, y)`

Renders a connected line chart to the terminal.

- `x` — row or column vector of x-coordinates
- `y` — row or column vector of y-coordinates (same length as `x`)

```
>> plot(1:10, (1:10).^2)
```

### `scatter(x, y)`

Renders individual data points (no connecting lines).

```
>> scatter([-2:0.5:2], ([-2:0.5:2]).^3)
```

### `title("text")`

Sets the chart title printed above the next `plot` or `scatter` call.

```
>> title("Parabola")
>> plot(1:5, (1:5).^2)
```

### `xlabel("text")`

Prints a label for the x-axis below the chart.

### `ylabel("text")`

Prints a label for the y-axis below the chart.

## Annotation state

`xlabel`, `ylabel`, and `title` store their arguments in a thread-local
`FigureState`. The state is consumed and **reset** by the next `plot` or
`scatter` call, so annotations must be set before each plot:

```
title("My Chart")
xlabel("time (s)")
ylabel("amplitude")
plot(t, y)
```

## Feature flags

| Feature      | Enables                        |
|--------------|--------------------------------|
| `plot`       | ASCII terminal rendering       |
| `plot-svg`   | SVG/PNG file output (Phase 29b)|
| `plot-all`   | Both of the above              |

Without the `plot` feature, calling `plot(...)` returns an informative error
suggesting the rebuild command.

## See also

- [Plugins](./plugins.md) — how the `ccalc-plot` plugin is registered
- [Phase 29 — Plot engine](../ccalc/phase29-plot.md) — implementation notes
