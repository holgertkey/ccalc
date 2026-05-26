# Configuration

ccalc stores persistent settings in a plain-text TOML file:

```
~/.config/ccalc/config.toml          # Linux / macOS
%APPDATA%\ccalc\config.toml          # Windows
```

The file is created automatically with defaults the first time you start the
interactive REPL. You can edit it with any text editor.

## Default config.toml

```toml
# ccalc configuration
# Edit this file and run 'config reload' in the REPL to apply changes.

[display]
# Default decimal precision (number of digits after the decimal point, 0–15).
precision = 10

# Default number base for output: "dec", "hex", "bin", "oct"
base = "dec"

[repl]
# Prompt templates — see the "Prompt customization" section below.
# prompt1 = "[ {ans} ]: "
# prompt2 = "  >> "

[highlight]
# Set to false to disable real-time syntax highlighting.
enabled = true
# Uncomment and set to override default colours.  Formats: named ("yellow"),
# 8-bit ("color256(220)"), truecolor ("#FFD700"), or "bold:<colour>".
# keywords = "yellow"
# numbers  = "cyan"
# strings  = "green"
# comments = "dark_gray"
# builtins = "bright_cyan"
# errors   = "red"
```

## Settings

### `display.precision`

Number of decimal places shown in the output. Range: 0–15. Default: `10`.

Values above 15 are silently clamped to 15.

This is the same value controlled by `p<N>` during a session. Changes in
config take effect on the next REPL start (or after `config reload`).

### `display.base`

Default output base. Accepted values: `"dec"`, `"hex"`, `"bin"`, `"oct"`.
Default: `"dec"`.

Unknown values fall back to `"dec"` without error.

## Prompt customization

The `[repl]` section lets you set custom prompt templates for `prompt1` (the
primary prompt, shown when ready for new input) and `prompt2` (the secondary
prompt, shown inside multi-line blocks such as `if`/`for`/`while`).

```toml
[repl]
prompt1 = "[ {ans} ]: "    # default
prompt2 = "  >> "           # default
```

### Content placeholders

| Placeholder | Expands to |
|-------------|------------|
| `{ans}` | Formatted value of `ans` — the default prompt content |
| `{line}` | Session command counter (increments after each input) |
| `{user}` | Current OS username |
| `{host}` | Short hostname (before the first dot) |
| `{cwd}` | Full current working directory |
| `{cwd_short}` | Last path component of the current directory |
| `{time}` | Current time as `HH:MM:SS` (UTC) |

### Color placeholders

Color codes are emitted only for the displayed prompt and do not affect cursor
positioning. Any number of color/style placeholders can be combined.

| Placeholder | Effect |
|-------------|--------|
| `{reset}` | Turn off all colour/style |
| `{bold}` | Bold text |
| `{dim}` | Dim/faint text |
| `{black}` | Black foreground |
| `{red}` | Red foreground |
| `{green}` | Green foreground |
| `{yellow}` | Yellow foreground |
| `{blue}` | Blue foreground |
| `{magenta}` | Magenta foreground |
| `{cyan}` | Cyan foreground |
| `{white}` | White foreground |
| `{gray}` | Bright black (dark gray) foreground |
| `{bright_red}` | Bright red foreground |
| `{bright_green}` | Bright green foreground |
| `{bright_yellow}` | Bright yellow foreground |
| `{bright_blue}` | Bright blue foreground |
| `{bright_magenta}` | Bright magenta foreground |
| `{bright_cyan}` | Bright cyan foreground |
| `{bright_white}` | Bright white foreground |
| `{#RRGGBB}` | 24-bit truecolor foreground (e.g. `{#FF8800}` for orange) |

### Examples

```toml
[repl]
# Minimal: show counter and ans
prompt1 = "{line} [ {ans} ]: "

# Counter dimmed, ans in default colour
prompt1 = "{gray}({line}){reset} [ {ans} ]: "

# Shell-style: user@host:dir$
prompt1 = "{green}{user}@{host}{reset}:{cyan}{cwd_short}{reset}$ "

# Bold blue name, dimmed counter, ans
prompt1 = "{bold}{blue}ccalc{reset} {gray}[{line}]{reset} {ans} > "

# 24-bit orange accent colour
prompt1 = "{#FF8800}ccalc{reset} [{line}] {ans} > "
```

After editing `config.toml`, apply changes without restarting:

```
[ 0 ]: config reload
Config reloaded.
```

## Syntax highlighting

The `[highlight]` section controls real-time input highlighting in the REPL.

```toml
[highlight]
enabled = true      # set to false to disable highlighting entirely

# Colour formats:
#   Named 4-bit  — black, red, green, yellow, blue, magenta, cyan, white
#                  bright_black (dark_gray), bright_red, bright_green,
#                  bright_yellow, bright_blue, bright_magenta,
#                  bright_cyan, bright_white
#   8-bit        — color256(N)  where N = 0..255
#   True color   — #RRGGBB     (hex, requires a true-color terminal)
#
# Prefix any value with "bold:" for bold text, e.g. "bold:yellow"

# keywords = "yellow"
# numbers  = "cyan"
# strings  = "green"
# comments = "dark_gray"
# builtins = "bright_cyan"
# errors   = "red"
```

### Colour categories

| Key | Default | Highlighted tokens |
|-----|---------|-------------------|
| `keywords` | yellow | `if`, `for`, `while`, `end`, `function`, `else`, `elseif`, `return`, `break`, `continue`, `do`, `until`, `switch`, `case`, `otherwise`, `try`, `catch`, `global`, `persistent` |
| `numbers` | cyan | Integer, decimal, scientific, and hex literals (`42`, `3.14`, `1e-3`, `0xFF`) |
| `strings` | green | Single-quoted `'...'` and double-quoted `"..."` string literals |
| `comments` | dark gray | `%` and `#` to end of line |
| `builtins` | bright cyan | All built-in function names (`sin`, `plot`, `zeros`, …) and plugin functions |
| `errors` | red | Unclosed string literals or brackets |

User-defined variables and operators are shown in the terminal's default colour.

### Shadowing rules

If a name from a keyword or built-in list is assigned as a variable (e.g. `end = 42`),
the highlighting uses default colour for that name — matching evaluation semantics.

### Colour format reference

| Format | Example | Notes |
|--------|---------|-------|
| Named 4-bit | `"yellow"`, `"bright_cyan"` | 16 standard terminal colours |
| 8-bit palette | `"color256(220)"` | 256-colour extended palette |
| 24-bit truecolor | `"#FFD700"` | Requires a true-color terminal |
| Bold prefix | `"bold:yellow"` | Combines bold with any colour |

Unknown values are silently ignored and the built-in default is used instead.

## REPL commands

| Command | Action |
|---|---|
| `config` | Show config file path and currently active settings |
| `config reload` | Re-read `config.toml` and apply changes immediately |

Changes made with `p<N>`, `hex`, `dec`, `bin`, `oct` during a session are
session-local and are **not** written back to `config.toml`.

## Example

Set precision to 4 and default base to hex, then apply without restarting:

1. Edit `config.toml`:
   ```toml
   [display]
   precision = 4
   base = "hex"
   ```
2. In the REPL:
   ```
   [ 0 ]: config reload
   Config reloaded.
   precision:   4
   base:        hex
   [ 0x0 ]:
   ```

## Config file location

The `config` command shows the full path of the file on the current system:

```
[ 0 ]: config
config file: /home/user/.config/ccalc/config.toml
precision:   10
base:        dec
```
