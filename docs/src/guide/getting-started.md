# Getting Started

## Installation

Build from source (requires [Rust](https://rustup.rs/)):

```sh
git clone https://github.com/holgertkey/ccalc
cd ccalc
cargo build --release
# binary: target/release/ccalc
```

## Three usage modes

### Interactive REPL

```sh
ccalc
```

A prompt shows the current accumulator value. Type an expression and press Enter.

```
[ 0 ]: 2 ^ 32
[ 4294967296 ]: / 1024
[ 4194304 ]: sqrt()
[ 2048 ]: q
```

### Single expression (argument mode)

```sh
ccalc "EXPR"
```

Evaluates the expression, prints the result, and exits. Useful for shell scripts.

```sh
$ ccalc "2 ^ 32"
4294967296

$ ccalc "sqrt(2)"
1.4142135624
```

### Pipe / non-interactive mode

When stdin is not a terminal, ccalc reads lines one by one, prints one result per
line, and carries the accumulator across lines.

```sh
$ echo "sin(pi / 6)" | ccalc
0.5

$ printf "100\n/ 4\n+ 5" | ccalc
100
25
30
```

## Command-line options

| Flag | Description |
|---|---|
| `-h`, `--help` | Print help and exit |
| `-v`, `--version` | Print version and exit |
