# REPL Mode

Start with `ccalc` (no arguments, stdin is a terminal).

## Prompt

The prompt always shows the current value of `ans`:

```
[ 0 ]:
[ 42 ]:
[ 0xFF ]:
```

## ans

Every expression result is stored in `ans`. Expressions that start with
an operator use `ans` as the left-hand operand (**partial expressions**):

```
[ 0 ]: 100
[ 100 ]: * 2
[ 200 ]: + 50
[ 250 ]: / 5
[ 50 ]:
```

## REPL commands

| Command | Action |
|---|---|
| `exit`, `quit` | Quit |
| `cls` | Clear the screen |
| `help`, `?` | Show cheatsheet |
| `help <topic>` | Detailed help: `syntax` `functions` `bases` `vars` `script` `examples` |
| `who` | Show all defined variables |
| `clear` | Clear all variables |
| `clear <name>` | Clear a single variable |
| `p` | Show current decimal precision |
| `p<N>` | Set precision to N decimal places (0–15) |
| `hex` / `dec` / `bin` / `oct` | Switch display base |
| `base` | Show ans in all four bases |
| `ws` | Save workspace to file |
| `wl` | Load workspace from file |

## Keyboard shortcuts

| Key | Action |
|---|---|
| `↑` / `↓` | Browse input history |
| `Ctrl+R` | Reverse history search |
| `← → Home End` | Cursor movement |
| `Ctrl+W` | Delete word before cursor |
| `Ctrl+U` | Clear line |
| `Ctrl+C` / `Ctrl+D` | Quit |

## Silencing a line

Append `;` to suppress output while still updating `ans`:

```
[ 0 ]: 0.06 / 12;
[ 0.005 ]:
```

The prompt updates, but no result is printed on that line.

## History

Input history is saved to `~/.config/ccalc/history` and restored on the next
session.
