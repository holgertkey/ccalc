# REPL Mode

Start with `ccalc` (no arguments, stdin is a terminal).

## Prompt

The prompt always shows the current accumulator:

```
[ 0 ]:
[ 42 ]:
[ 0xFF ]:
```

## Accumulator

Every expression result becomes the new accumulator. Expressions that start with
an operator use the accumulator as the left-hand operand (**partial expressions**):

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
| `q` | Quit |
| `c` | Clear accumulator (reset to 0) |
| `cls` | Clear the screen |
| `p` | Show current decimal precision |
| `p<N>` | Set precision to N decimal places (0–15) |
| `hex` / `dec` / `bin` / `oct` | Switch display base |
| `base` | Show accumulator in all four bases |
| `m` | Show all non-zero memory cells |
| `mc` | Clear all memory cells |
| `ms` | Save memory cells to file |
| `ml` | Load memory cells from file |

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

Append `;` to suppress output while still updating the accumulator:

```
[ 0 ]: 0.06 / 12;
[ 0.005 ]:
```

The prompt updates, but no result is printed on that line.

## History

Input history is saved to `~/.config/ccalc/history` and restored on the next
session.
