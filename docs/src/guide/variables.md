# Variables

ccalc supports named variables. Any valid identifier can store a value.

## Assignment

Use `name = expr` to assign:

```
[ 0 ]: rate = 0.06 / 12
rate = 0.005
[ 0.005 ]: n = 360
n = 360
```

The result is printed as `name = value`. Assignment also updates `ans`.

Append `;` to assign silently:

```
rate = 0.06 / 12;
n = 360;
```

## Using variables

Any defined variable can appear inside an expression:

```
[ 0 ]: rate = 0.07
rate = 0.07
[ 0.07 ]: 1000 * (1 + rate) ^ 10
[ 1967.1513573 ]:
```

## ans

`ans` is the implicit result variable — set automatically after every expression that is not an assignment. It is initialized to `0` and reset to `0` by the `c` command.

Expressions starting with an operator use `ans` as the left-hand operand:

```
[ 0 ]: 100
[ 100 ]: / 4
[ 25 ]: + 5
[ 30 ]:
```

Empty-argument function calls use `ans` as the argument:

```
[ 144 ]: sqrt()      →  12     (same as sqrt(144))
```

## Constants

`pi` and `e` are pre-defined read-only constants:

| Name | Value |
|---|---|
| `pi` | 3.14159265358979… |
| `e`  | 2.71828182845904… |

## View and clear

| Command      | Action                                              |
|--------------|-----------------------------------------------------|
| `who`        | Show all defined variables and their values         |
| `clear`      | Clear all variables                                 |
| `clear name` | Clear a single variable by name                     |

```
[ 0 ]: x = 10
x = 10
[ 10 ]: y = 3.14
y = 3.14
[ 3.14 ]: who
ans = 3.14
x = 10
y = 3.14
[ 3.14 ]: clear x
[ 3.14 ]: who
ans = 3.14
y = 3.14
[ 3.14 ]: clear
```

## Workspace persistence

| Command | Action                                                    |
|---------|-----------------------------------------------------------|
| `ws`    | Save all variables to `~/.config/ccalc/workspace.toml`   |
| `wl`    | Load variables from file (replaces current workspace)     |

The workspace file is plain text, one `name = value` entry per line:

```
ans = 3.14
n = 360
rate = 0.005
```

## Example — monthly mortgage

```
rate = 0.06 / 12;
n = 360;
factor = (1 + rate) ^ n;
200000 * rate * factor / (factor - 1)
print "Monthly payment ($):"
```

Output:

```
Monthly payment ($): 1199.1010503
```
