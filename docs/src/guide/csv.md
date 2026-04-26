# CSV — Tables and Matrices

ccalc provides three built-in functions for reading and writing delimiter-separated
files. They extend the lower-level [`dlmread`/`dlmwrite`](./file-io.md) primitives
with automatic header handling, mixed-type columns, and RFC 4180 quoting.

---

## `readmatrix`

`readmatrix(path)` reads a numeric CSV file and returns a `Matrix`.

```matlab
A = readmatrix('data.csv')
```

**Behaviour:**

- Auto-detects the delimiter: comma → tab → whitespace.
- If the first row contains any non-numeric text it is skipped as a header.
  A purely numeric first row is treated as data.
- Empty cells become `NaN` (unlike `dlmread`, which uses `0.0`).

```matlab
% data.csv:  x,y,z\n1,2,3\n4,5,6
A = readmatrix('data.csv')
% → [1 2 3; 4 5 6]  (header row skipped)
```

**Explicit delimiter:**

```matlab
A = readmatrix('data.tsv', 'Delimiter', '\t')
```

---

## `readtable`

`readtable(path)` reads a CSV file where the **first row is always the header**
and returns a `Struct` of columns.

```matlab
T = readtable('people.csv')
T.name    % Cell of Str — one element per row
T.age     % Matrix (N×1) — numeric column
```

**Column type rules:**

| Column content | ccalc type |
|----------------|-----------|
| All cells parseable as numbers (empty → `NaN`) | `Matrix` N×1 |
| Any non-numeric cell | `Cell` of `Str` |

**Quoted fields (RFC 4180):**

Fields may be enclosed in double-quotes. A comma inside a quoted field is part
of the value, not a delimiter. Two consecutive `""` inside a quoted field encode
a literal `"`.

```matlab
% people.csv:
%   name,city
%   "Smith, John","New York"
T = readtable('people.csv')
T.name{1}   % → 'Smith, John'
T.city{1}   % → 'New York'
```

**Explicit delimiter:**

```matlab
T = readtable('data.tsv', 'Delimiter', '\t')
```

---

## `writetable`

`writetable(T, path)` writes a struct table to a CSV file with a header row.

```matlab
T.name  = {'Alice'; 'Bob'};
T.score = [95; 87];
writetable(T, 'output.csv')
```

Output:
```
name,score
Alice,95
Bob,87
```

**Accepted column types:**

| ccalc type | Written as |
|-----------|-----------|
| `Matrix` (N×1) | One number per row |
| `Cell` | Each element formatted (strings or numbers) |
| `Scalar` | Single-row value |
| `Str` / `StringObj` | Single-row string |

**Quoting:** any cell value that contains the delimiter, a `"`, or a newline
is automatically wrapped in double-quotes (RFC 4180). Embedded `"` are doubled.

```matlab
T.desc = {'hello, world'; 'plain'};
T.n    = [1; 2];
writetable(T, 'out.csv')
% out.csv:
%   desc,n
%   "hello, world",1
%   plain,2
```

**Explicit delimiter:**

```matlab
writetable(T, 'out.tsv', 'Delimiter', '\t')
```

---

## Roundtrip example

```matlab
% Write
T.city  = {'Paris'; 'Berlin'; 'Tokyo'};
T.pop   = [2161000; 3645000; 13960000];
writetable(T, 'cities.csv')

% Read back
T2 = readtable('cities.csv')
T2.city{2}   % → 'Berlin'
T2.pop(3)    % → 13960000
```

---

## Differences from `dlmread` / `dlmwrite`

| Feature | `dlmread` | `readmatrix` | `readtable` |
|---------|-----------|--------------|-------------|
| Header row | error | auto-skip | always first row |
| Empty cells | `0.0` | `NaN` | `NaN` (numeric cols) |
| String columns | error | error | `Cell` |
| Quoted fields | no | yes | yes |
| Return type | `Matrix` | `Matrix` | `Struct` of columns |
