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
