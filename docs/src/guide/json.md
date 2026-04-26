# JSON

ccalc can encode and decode JSON data using two built-in functions. These functions
are available when ccalc is built with the `json` feature flag.

> **Requires the `json` feature:**
> ```bash
> cargo build --release --features json
> ```
> Without this flag, calling `jsondecode` or `jsonencode` returns an error message
> explaining how to enable the feature.

---

## Decoding JSON

`jsondecode(str)` parses a JSON string and returns a ccalc value.

```matlab
s = jsondecode('{"x": 1, "y": [1, 2, 3]}')
% s is a struct with fields x and y
s.x      % → 1
s.y      % → [1 2 3]  (1×3 matrix row vector)
```

### Type mapping

| JSON type | ccalc value |
|-----------|-------------|
| object `{…}` | `Struct` |
| all-numeric array | `Matrix` (1×N row vector) |
| mixed array | `Cell` |
| string | `Str` (char array) |
| number | `Scalar` |
| `true` / `false` | `Scalar` (1 / 0) |
| `null` | `Scalar(NaN)` |

Arrays containing only numbers (and `null` values, which become `NaN`) decode to
a `Matrix` row vector. Arrays with mixed types (numbers, strings, nested objects,
etc.) decode to a `Cell`.

```matlab
jsondecode('[1, 2, 3]')          % → [1 2 3]  (Matrix)
jsondecode('[1, "two", 3]')      % → {1, 'two', 3}  (Cell)
jsondecode('null')               % → NaN
jsondecode('true')               % → 1
```

### Nested data

Nested JSON objects become nested structs:

```matlab
data = jsondecode('{"person": {"name": "Alice", "age": 30}}');
data.person.name   % → 'Alice'
data.person.age    % → 30
```

### Reading from a file

Combine with `fileread` to decode a JSON file:

```matlab
raw = fileread('data.json');
data = jsondecode(raw);
```

---

## Encoding JSON

`jsonencode(val)` encodes a ccalc value to a compact JSON string.

```matlab
s.x = 1;
s.y = [1 2 3];
jsonencode(s)   % → '{"x":1.0,"y":[1.0,2.0,3.0]}'
```

### Type mapping

| ccalc value | JSON output |
|-------------|-------------|
| `Struct` | object `{…}` |
| `Matrix` (1×N row vector) | flat array `[…]` |
| `Matrix` (M×N) | array of row arrays |
| `Cell` | array `[…]` |
| `Scalar` | number |
| `Scalar(NaN)` | `null` |
| `Str` / `StringObj` | string |

`Scalar(Inf)` and `Scalar(-Inf)` cannot be represented in JSON and produce an error.
`Complex`, `Lambda`, and `Function` values also produce an error.

```matlab
jsonencode(42)            % → '42.0'
jsonencode('hello')       % → '"hello"'
jsonencode([1 2 3])       % → '[1.0,2.0,3.0]'
jsonencode({1, 'a'})      % → '[1.0,"a"]'
```

### Writing to a file

```matlab
s.result = 3.14;
fid = fopen('output.json', 'w');
fprintf(fid, '%s\n', jsonencode(s));
fclose(fid);
```

---

## Roundtrip example

```matlab
original = '{"name":"Bob","scores":[88,92,75]}';
data = jsondecode(original);
data.name       % → 'Bob'
data.scores     % → [88 92 75]

% Re-encode (field order preserved via IndexMap):
jsonencode(data)
% → '{"name":"Bob","scores":[88.0,92.0,75.0]}'
```
