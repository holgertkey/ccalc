# Number Bases

## Input literals

Any numeric literal can be written in hex, binary, or octal:

| Prefix | Base | Example | Value |
|---|---|---|---|
| `0x` / `0X` | Hexadecimal | `0xFF` | 255 |
| `0b` / `0B` | Binary | `0b1010` | 10 |
| `0o` / `0O` | Octal | `0o17` | 15 |

Mixed-base expressions work naturally:

```
0xFF + 0b1010        →  265
0x10 + 0o10 + 0b10   →   26
```

## Display base

By default results are shown in decimal. Switch with a command (session-local):



| Command | Effect |
|---|---|
| `dec` | Decimal (default) |
| `hex` | Hexadecimal |
| `bin` | Binary |
| `oct` | Octal |

The display base persists until changed and affects both the prompt and all results.
To make a non-decimal base the default across all sessions, set it in
[`config.toml`](./configuration.md).

```
[ 0 ]: 255
[ 255 ]: hex
[ 0xFF ]: + 1
[ 0x100 ]: dec
[ 256 ]:
```

## Inline base suffix

Write a base keyword after an expression to evaluate it **and** switch the display
in one step:

```
[ 0 ]: 0xFF + 0b1010 hex
[ 0x109 ]:
```

## `base` — show all representations

```
[ 10 ]: base
2  - 0b1010
8  - 0o12
10 - 10
16 - 0xA
```

`base` can also be used as an inline suffix to show one result in all four bases
without changing the active display base:

```
[ 0 ]: 255 base
2  - 0b11111111
8  - 0o377
10 - 255
16 - 0xFF
[ 255 ]:
```

## Mixed-base display conversion

When the active display base is non-decimal and the input contains literals in
other bases, the expression is automatically reprinted with all literals converted
to the active base before the result is shown:

```
[ 0b110 ]: 2 + 0b110 + 0xa
0b10 + 0b110 + 0b1010
[ 0b10010 ]:
```
