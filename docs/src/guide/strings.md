# Strings

ccalc supports two string types that match MATLAB/Octave:

| Type | Syntax | Semantic |
|------|--------|----------|
| Char array | `'single quotes'` | 1×N array of characters, numeric-compatible |
| String object | `"double quotes"` | Scalar string, concatenation with `+` |

---

## Char arrays

A char array is a **1×N row of character codes**. Single quotes delimit it.
Inside a char array, `''` (two consecutive single quotes) represents a
literal single quote character.

```
[ 0 ]: s = 'Hello'
s = Hello
[ 'Hello' ]: length(s)
[ 5 ]: size(s)
ans =
   1   5
```

### Arithmetic — characters as ASCII codes

Char arrays convert to their ASCII codes before any arithmetic operation.
The result is a numeric scalar or row vector, not a string.

```
[ 0 ]: 'A' + 0        % ASCII code of 'A'
[ 65 ]:
[ 0 ]: 'a' + 1        % shift by one position
[ 98 ]:
[ 0 ]: 'abc' + 0      % codes for 'a', 'b', 'c'
ans =
   97   98   99
[ 0 ]: 'abc' + 1      % shift every character
ans =
   98   99   100
```

Element-wise comparison returns a 0/1 row vector:

```
[ 0 ]: 'abc' == 'aXc'
ans =
   1   0   1
```

### Escaped single quote

Use `''` inside a char array to include a literal `'`:

```
[ 0 ]: disp('it''s fine')
it's fine
```

### Char-array concatenation with `[...]`

The bracket operator concatenates char arrays horizontally — the standard
MATLAB/Octave idiom for building strings dynamically:

```matlab
['hello' ' world']          % → 'hello world'
['a' 'b' 'c']               % → 'abc'
['prefix_' num2str(k)]      % → 'prefix_3'  (when k = 3)
```

**String context** (first element is a char array): numeric elements are
treated as Unicode code points and become characters.

```matlab
['A' 66 67]                 % → 'ABC'  (65='A', 66='B', 67='C')
['A' [66 67]]               % → 'ABC'  (matrix of codes)
```

**Numeric context** (first element is numeric): char-array elements
contribute their code values.

```matlab
[65 'B']                    % → [65 66]
[1 'AB']                    % → [1 65 66]
```

**Note:** each space before a `'` signals the start of a new string literal,
matching MATLAB whitespace-aware disambiguation. `['a' 'b']` with a space
between the two char arrays correctly produces `'ab'`.

---

## String objects

A string object is a **scalar container** — one string, not a character-by-character array.
Double quotes delimit it. `""` inside a string object represents a literal `"`.
Backslash escape sequences work: `\n`, `\t`, `\\`, `\"`.

```
[ 0 ]: t = "Hello"
t = Hello
[ '"Hello"' ]: t + ", World!"
[ '"Hello, World!"' ]:
```

`length` and `numel` return `1` (it is a 1×1 scalar string):

```
[ 0 ]: length("hello")
[ 1 ]: numel("hello")
[ 1 ]: size("hello")
ans =
   1   1
```

### Concatenation with `+`

```
[ 0 ]: "foo" + "bar"
[ '"foobar"' ]:
[ 0 ]: a = "left"; b = " right";
[ 0 ]: a + b
[ '"left right"' ]:
```

### Comparison

`==` and `~=` compare entire string objects:

```
[ 0 ]: "hello" == "hello"
[ 1 ]:
[ 0 ]: "hello" == "world"
[ 0 ]:
[ 0 ]: "abc" ~= "ABC"
[ 1 ]:
```

---

## Type checks

```
[ 0 ]: ischar('hello')    % 1 — it's a char array
[ 1 ]:
[ 0 ]: isstring("hello")  % 1 — it's a string object
[ 1 ]:
[ 0 ]: ischar("hello")    % 0 — string object is NOT a char array
[ 0 ]:
[ 0 ]: ischar(42)         % 0
[ 0 ]:
```

---

## String built-ins

### Number conversions

```
[ 0 ]: num2str(42)
42
[ 0 ]: num2str(3.14159)
3.1416
[ 0 ]: num2str(3.14159, 2)    % 2 decimal digits
3.14
[ 0 ]: str2double('2.718')
[ 2.718 ]:
[ 0 ]: str2double('abc')      % NaN on failure
[ NaN ]:
[ 0 ]: str2num('100')
[ 100 ]:
```

### Concatenation

`strcat` works on both char arrays and string objects:

```
[ 0 ]: strcat('foo', 'bar')
foobar
[ 0 ]: strcat("unit: ", num2str(42), " Hz")
unit: 42 Hz
```

### Comparison functions

```
[ 0 ]: strcmp('abc', 'abc')     % 1 — case-sensitive equal
[ 1 ]:
[ 0 ]: strcmp('abc', 'ABC')     % 0
[ 0 ]:
[ 0 ]: strcmpi('abc', 'ABC')    % 1 — case-insensitive
[ 1 ]:
```

### Case and whitespace

```
[ 0 ]: upper('hello')
HELLO
[ 0 ]: lower('WORLD')
world
[ 0 ]: strtrim('  spaces  ')
spaces
```

### Search and replace

```
[ 0 ]: strrep('the cat sat', 'cat', 'dog')
the dog sat
[ 0 ]: strrep("Hello World", "World", "ccalc")
Hello ccalc
```

### Predicates — containment, prefix, suffix

```
[ 0 ]: contains('hello world', 'world')
[ 1 ]:
[ 0 ]: contains('hello', 'xyz')
[ 0 ]:
[ 0 ]: contains('Hello', 'hello', 'IgnoreCase', true)
[ 1 ]:
[ 0 ]: startsWith('hello', 'he')
[ 1 ]:
[ 0 ]: endsWith('hello', 'lo')
[ 1 ]:
```

### Splitting and joining strings

`strsplit` splits a string on a delimiter and returns a **cell array** of char arrays:

```
[ 0 ]: parts = strsplit('alpha,beta,gamma', ',')
[ 0 ]: numel(parts)
[ 3 ]:
[ 0 ]: parts{1}
alpha
[ 0 ]: parts{2}
beta
```

Without a delimiter, `strsplit` splits on whitespace:

```
[ 0 ]: words = strsplit('hello world')
[ 0 ]: words{1}
hello
```

`strjoin` is the inverse — it joins a cell array of strings into one string:

```
[ 0 ]: strjoin({'a', 'b', 'c'}, ',')
a,b,c
[ 0 ]: strjoin({'x', 'y'})
x y
```

### Integer and matrix string conversion

```
[ 0 ]: int2str(3.2)          % round to nearest integer, return string
3
[ 0 ]: int2str(3.7)
4
[ 0 ]: int2str(-1.5)
-2

[ 0 ]: mat2str([1 2; 3 4])   % matrix → MATLAB literal syntax
[1 2;3 4]
[ 0 ]: mat2str([10 20 30])
[10 20 30]
```

### `sprintf`

Single-argument form: returns a char array with escape sequences processed.

```
[ 0 ]: disp(sprintf('line 1\nline 2\n'))
line 1
line 2

[ 0 ]: disp(sprintf('A\tB\tC'))
A	B	C
```

---

## Regular expressions

Regular expression support is available when ccalc is built with
`--features regex`. Without the feature, calling these functions returns
an informative error message. Both names are always available for tab
completion.

```
% Find start index of first match (1-based); [] if no match:
regexp('abc 123 def', '\d+')          % → 5

% Extract all matched substrings as a cell array:
regexp('abc 123 def 456', '\d+', 'match')   % → {'123', '456'}

% Case-insensitive search:
regexpi('Hello World', 'hello')       % → 1

% Replace all matches (replacement is always a literal string):
regexprep('foo  bar', '\s+', '_')     % → 'foo_bar'
regexprep('2024-01-15', '-', '/')     % → '2024/01/15'
regexprep('a', 'a', '$1')            % → '$1'  (not expanded)
```

Build with regex support:

```
cargo build --features regex
```

---

## Displaying strings

String values display as plain text — no surrounding quotes in the output:

```
[ 0 ]: 'hello'
hello
[ 0 ]: "world"
world
[ 0 ]: x = strcat('value: ', num2str(42))
x = value: 42
```

The REPL prompt shows the string content (truncated at 15 characters) when
`ans` is a string.

`who` annotates string types:

```
[ 0 ]: s = 'abc'; t = "hello";
[ 0 ]: who
Variables visible from the current scope:

ans = 0
s [1×3 char]
t [string]
```

---

## Workspace

`ws` and `wl` do **not** persist string variables — the same policy as
matrices and complex numbers. Only scalars are saved.

---

## Practical example — labelled output

```
R = 4700;
C = 2.2e-9;
f0 = 1 / (2 * pi * R * C);

fprintf('RC filter\n')
fprintf('  R  = ')
disp(strcat(num2str(R), ' Ohm'))
fprintf('  C  = ')
disp(strcat(num2str(C * 1e9, 3), ' nF'))
fprintf('  f0 = ')
disp(strcat(num2str(f0, 5), ' Hz'))
```

Output:

```
RC filter
  R  = 4700 Ohm
  C  = 2.2 nF
  f0 = 15392 Hz
```

See `examples/strings.calc` for the full demo: `ccalc examples/strings.calc`
