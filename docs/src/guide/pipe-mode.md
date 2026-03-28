# Pipe & Script Mode

When stdin is not a terminal (pipe or file redirect), ccalc runs in
non-interactive mode: no prompts, one result printed per line.

## Pipe

```sh
echo "1 + 1" | ccalc
printf "100\n/ 4\n+ 5" | ccalc
```

## Script files

```sh
ccalc < formula.txt
```

### Comments

```
# full-line comment — line is skipped entirely
10 * 5  # inline comment — expression still evaluates
```

### Semicolon — suppress output

A trailing `;` evaluates the line and updates the accumulator, but prints nothing:

```
0.06 / 12;       # compute monthly rate, silently
m1;              # store it, silently
1 + m1           # this line is printed
```

### `print` — explicit output

```
print              # print current accumulator value
print "label"      # print label then value
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

All REPL commands except `cls` and `m` (which are ignored):
`q`, `c`, `mc`, `mc[1-9]`, `m[1-9]`, `p`, `p<N>`, `hex`, `dec`, `bin`, `oct`, `base`.

## Example — mortgage script

```
# Monthly mortgage payment
# Principal: 200 000, annual rate: 6%, term: 30 years

0.06 / 12;             # monthly rate r
m1;
1 + m1;                # (1 + r)
^ 360;                 # (1 + r)^360
m2;
200000 * m1 * m2;      # numerator
m3;
m3 / (m2 - 1)          # monthly payment
print "Monthly payment ($):"
```
