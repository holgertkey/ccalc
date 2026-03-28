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

A trailing `;` evaluates the line and updates `ans`, but prints nothing:

```
rate = 0.06 / 12;    % compute monthly rate, silently
n = 360;             % term in months, silently
factor = (1 + rate) ^ n;
```

### `print` — explicit output

```
print              % print current ans value
print "label"      % print label then value
```

The label is the full quoted string. Write any punctuation you want inside it:

```
print "Result:"    →  Result: 42
print "Sum ="      →  Sum = 42
```

**Section headers** — `print "label"` placed right after a blank line (or at the
very start of the file) prints the label only, without a value:

```
print "=== Monthly mortgage ==="

200000 * 0.005
print "First payment:"
```

Output:
```
=== Monthly mortgage ===
1000
First payment: 1000
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
p2;

p * rate * factor / (factor - 1);
print "Monthly payment ($):"
```
