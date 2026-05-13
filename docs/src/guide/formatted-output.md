# Formatted Output

ccalc supports C-style formatted output via `fprintf` and `sprintf`, matching
Octave/MATLAB semantics.

## `fprintf` — print to stdout

```
fprintf(fmt, v1, v2, ...)
```

Prints formatted text to stdout. No return value — result display is suppressed.
No newline is added automatically; include `\n` explicitly.

```
fprintf('pi = %.4f\n', pi)         % pi = 3.1416
fprintf('n = %d items\n', 42)      % n = 42 items
```

## `sprintf` — format to string

```
s = sprintf(fmt, v1, v2, ...)
```

Same format engine as `fprintf`, but returns the result as a char array instead
of printing it.

```
label = sprintf('R = %.1f Ohm', 47.5);
disp(label)      % R = 47.5 Ohm
```

## Format specifiers

| Specifier  | Meaning                                          |
|------------|--------------------------------------------------|
| `%d`, `%i` | Integer (value truncated to whole number)        |
| `%f`       | Fixed-point decimal, default 6 places            |
| `%e`       | Scientific notation (`1.23e+04`)                |
| `%g`       | Shorter of `%f` and `%e`                        |
| `%x`       | Hexadecimal, lowercase (`ff`)                    |
| `%X`       | Hexadecimal, uppercase (`FF`)                    |
| `%s`       | String (char array or string object)             |
| `%%`       | Literal `%`                                      |

## Width, precision, and flags

The general form is:

```
%[flags][width][.precision]specifier
```

| Flag | Meaning                                       |
|------|-----------------------------------------------|
| `-`  | Left-align within field width                 |
| `+`  | Always show sign (+ or −)                     |
| `0`  | Zero-pad to field width                       |
| ` `  | Space in place of `+` for non-negative values |

Examples:

```
fprintf('%8.3f\n',   pi)     %      3.142
fprintf('%-10s|\n', 'hi')    % hi        |
fprintf('%+.4e\n', 1000)     % +1.0000e+03
fprintf('%05d\n',    42)     % 00042
fprintf('% d\n',      5)     %  5
fprintf('%x\n',     255)     % ff
fprintf('%04X\n',   255)     % 00FF
```

## Escape sequences

| Sequence | Character        |
|----------|------------------|
| `\n`     | Newline          |
| `\t`     | Tab              |
| `\\`     | Backslash        |

## Multiple arguments and repeat behaviour

When there are more arguments than conversion specifiers in the format string,
the format string repeats for the remaining arguments (Octave behaviour):

```
fprintf('%d\n', 1, 2, 3)
% 1
% 2
% 3

fprintf('x=%.1f  y=%.1f\n', 1, 2, 3, 4)
% x=1.0  y=2.0
% x=3.0  y=4.0
```

## Formatted data table example

```
fprintf('%6s %12s %12s\n', 'time', 'position', 'velocity')
fprintf('%6s %12s %12s\n', '(s)', '(m)', '(m/s)')
fprintf('%s\n', repmat('-', 1, 32))

t = 0:0.5:2;
pos = 0.5 * 9.81 * t.^2;
vel = 9.81 * t;

for k = 1:length(t)
  fprintf('%6.1f %12.3f %12.3f\n', t(k), pos(k), vel(k))
end
```

Output:
```
  time     position     velocity
   (s)          (m)        (m/s)
--------------------------------
   0.0        0.000        0.000
   0.5        1.226        4.905
   1.0        4.905        9.810
   1.5       11.036       14.715
   2.0       19.620       19.620
```

## See also

- [`format` command](./precision.md) — controls default display format for `disp()` and assignment output
- `help io` — concise in-REPL reference
- `help script` — full format specifier reference
- `examples/formatted_output.calc` — runnable example covering all specifiers
