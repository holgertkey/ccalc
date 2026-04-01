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
| `cls` | Clear the screen (also `Ctrl+L`) |
| `help`, `?` | Show cheatsheet |
| `help <topic>` | Detailed help (see topic list below) |
| `who` | Show all defined variables |
| `clear` | Clear all variables |
| `clear <name>` | Clear a single variable |
| `p` | Show current decimal precision |
| `p<N>` | Set precision to N decimal places (0–15) |
| `hex` / `dec` / `bin` / `oct` | Switch display base |
| `base` | Show ans in all four bases |
| `ws` | Save workspace to file |
| `wl` | Load workspace from file |
| `disp(expr)` | Print value without updating `ans` |
| `fprintf('fmt')` | Print formatted string (`\n`, `\t`, `\\` supported) |

Help topics for `help <topic>`:
`syntax` `functions` `bases` `vars` `script` `matrices` `examples`

## Keyboard shortcuts

| Key | Action |
|---|---|
| `↑` / `↓` | Browse input history |
| `Ctrl+R` | Reverse history search |
| `← →` / `Home` / `End` | Cursor movement |
| `Ctrl+A` | Go to beginning of line |
| `Ctrl+E` | Go to end of line |
| `Ctrl+W` | Delete word before cursor |
| `Ctrl+U` | Delete from cursor to beginning of line |
| `Ctrl+K` | Delete from cursor to end of line |
| `Ctrl+L` | Clear screen |
| `Ctrl+C` / `Ctrl+D` | Quit |

## Silencing a line

Append `;` to suppress output. For **expressions**, `ans` is still updated.
For **assignments**, `ans` is never updated regardless of `;`.

```
[ 0 ]: 0.06 / 12;          % expression — ans updated, output suppressed
[ 0.005 ]: rate = 0.07;    % assignment — silent, ans unchanged
[ 0.005 ]:
```

Multiple `;`-separated statements on one line — all but the last are silent:

```
[ 0 ]: a = 1; b = 2; c = 3;    % all silent
[ 0 ]: a = 1; b = 2             % a = 1 silent, b = 2 shown
b = 2
[ 0 ]:
```

## History

Input history is saved to `~/.config/ccalc/history` and restored on the next
session. Each session is marked with a timestamp comment:

```
% --- Session: 2026-04-01 14:22:07 UTC ---
rate = 0.06 / 12
n = 360
% --- Session: 2026-04-01 15:10:44 UTC ---
hypot(3, 4)
```

The marker uses `%` so it is harmless if accidentally recalled and executed.
