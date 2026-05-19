# Parser (`parser.rs`)

The parser converts an input string into an `Expr` AST through two stages:
tokenization and recursive-descent parsing.

## Tokenizer

`tokenize(input)` produces a `Vec<Token>`. Token types:

```rust
enum Token {
    Number(f64),   // decimal, hex (0x), binary (0b), octal (0o), scientific
    Ident(String), // function names and constants: sqrt, pi, e, acc, …
    Plus, Minus, Star, Slash, Caret, Percent,
    LParen, RParen,
}
```

### Numeric literals

The tokenizer handles all four bases and scientific notation:

| Literal | Example |
|---|---|
| Decimal | `3.14`, `100` |
| Scientific | `1e5`, `2.5e-3`, `1E+10` |
| Hexadecimal | `0xFF`, `0X1A` |
| Binary | `0b1010`, `0B11` |
| Octal | `0o17`, `0O377` |

Scientific notation uses lookahead to avoid treating `e` (Euler's number) as
an exponent when it appears as a standalone identifier.

## Grammar

```
expr    = term ( ('+' | '-') term )*
term    = unary ( ('*' | '/' | '.*' | './' | implicit_mul) unary )*
unary   = ('-' | '+' | '~') unary | power     -- unary lower than power
power   = primary (('^' | '.^' | '**') unary)?  -- right-associative
primary = ident '(' expr? ')'        -- function call or index
        | '(' expr ')'               -- grouping
        | '[' matrix ']'             -- matrix literal
        | number | ident             -- literal or variable
        | primary '\''               -- postfix conjugate transpose (highest)
        | primary '.\'               -- postfix non-conjugate transpose
```

Precedence follows MATLAB/Octave: `'` (transpose) > `^`/`.^` > unary `-`/`~` > `*`/`/` > `+`/`-`.

### Implicit multiplication

`parse_term` detects an `LParen` token following a completed expression and
inserts a `*` without consuming an explicit operator token. This allows
`2(3 + 1)` and `(a)(b)`.

### Percentage (`%`) disambiguation

`%` is right-context-sensitive inside `parse_term`:

- If the next token can start an expression → **modulo** (`BinOp(Mod)`)
- Otherwise → **postfix percentage** (`BinOp(Mul, Number(acc / 100))`)

### Accumulator in parsing

`parse` accepts `accumulator: f64`. This value is:

- Substituted for `acc` identifiers
- Substituted for empty-argument function calls: `sqrt()` → `sqrt(acc)`
- Used for postfix `%`: `20%` → `20 * (acc / 100)`

## Entry points

```rust
// Full parse — returns Err if any tokens remain after the expression
pub fn parse(input: &str, accumulator: f64) -> Result<Expr, String>

// True if input starts with an operator (caller should prepend acc)
pub fn is_partial(input: &str) -> bool
```
