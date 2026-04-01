# Pipe & Script Mode

When stdin is not a terminal (pipe or file redirect), or when a script file is
passed as an argument, ccalc runs in non-interactive mode: no prompts, one
result printed per line.

## Pipe

```sh
echo "1 + 1" | ccalc
printf "100\n/ 4\n+ 5" | ccalc
```

## Script files

Pass a file as an argument:

```sh
ccalc script.m
ccalc examples/mortgage.ccalc
```

Or redirect stdin:

```sh
ccalc < formula.txt
```

### Comments

`%` starts a comment (Octave/MATLAB convention):

```
% full-line comment — line is skipped entirely
10 * 5  % inline comment — expression still evaluates
```

### Semicolon — suppress output

A trailing `;` suppresses output. **Expressions** still update `ans`;
**assignments** never update `ans` regardless of `;`.

```
rate = 0.06 / 12;    % silent assignment — ans unchanged
n = 360;             % silent assignment — ans unchanged
factor = (1 + rate) ^ n;
```

Multiple `;`-separated statements on one line are also supported:

```
a = 1; b = 2; c = 3;    % all silent
a = 1; b = 2            % a = 1 silent, b = 2 printed
```

### `disp(expr)` — print value

`disp(expr)` evaluates the expression and prints the result.
It does **not** update `ans`.

```
disp(ans)               % print current ans value
disp(rate * 12)         % print expression result
```

### `fprintf('fmt')` — print formatted text

`fprintf('fmt')` prints a string with escape sequences (`\n`, `\t`, `\\`).
No newline is added automatically — include `\n` explicitly.

```
fprintf('=== Monthly mortgage ===\n')
fprintf('Result: ')
disp(ans)
```

Output:
```
=== Monthly mortgage ===
Result: 1199.1010503
```

### Supported commands in pipe mode

All REPL commands except `cls` (which is ignored):
`exit`, `quit`, `who`, `clear`, `clear <name>`, `ws`, `wl`,
`p`, `p<N>`, `hex`, `dec`, `bin`, `oct`, `base`.

## Example — mortgage script

```
% Monthly mortgage payment
% Principal: 200 000, annual rate: 6%, term: 30 years

rate = 0.06 / 12;         % monthly interest rate
n = 360;                  % 30 years * 12 months
p = 200000;               % principal

factor = (1 + rate) ^ n;

p * rate * factor / (factor - 1)
fprintf('Monthly payment ($): ')
disp(ans)
```

Output:
```
1199.1010503
Monthly payment ($): 1199.1010503
```
