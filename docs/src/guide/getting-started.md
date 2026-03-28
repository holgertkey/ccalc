# Getting Started

## Installation

Build from source (requires [Rust](https://rustup.rs/)):

```sh
git clone https://github.com/holgertkey/ccalc
cd ccalc
cargo build --release
# binary: target/release/ccalc
```

## Usage modes

### Interactive REPL

```sh
ccalc
```

A prompt shows the current value of `ans`. Type an expression and press Enter.

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

### Script file

Pass a `.m` or `.ccalc` file as an argument:

```sh
ccalc script.m
ccalc examples/mortgage.ccalc
```

### Pipe / non-interactive mode

When stdin is not a terminal, ccalc reads lines one by one and prints one result
per line. `ans` carries over across lines.

```sh
$ echo "sin(pi / 6)" | ccalc
0.5

$ printf "100\n/ 4\n+ 5" | ccalc
100
25
30

$ ccalc < formula.txt
```

## Command-line options

| Flag | Description |
|---|---|
| `-h`, `--help` | Print help and exit |
| `-v`, `--version` | Print version and exit |
