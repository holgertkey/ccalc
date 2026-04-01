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
