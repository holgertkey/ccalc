# Structs and Struct Arrays

A **scalar struct** groups named fields into a single value. Each field can
hold any type — scalar, matrix, string, complex, cell array, or another struct.
Fields are stored in insertion order (MATLAB-compatible behaviour using an
ordered map).

---

## Creating structs

### Field assignment

Assign to `name.field` to create the struct and set the field in one step:

```matlab
pt.x = 3;
pt.y = 4;
pt.z = 0;
```

If the variable does not yet exist it is created as an empty struct and then
the field is added. Assigning to a non-existent nested path creates all
intermediate levels automatically:

```matlab
car.engine.hp = 190;       % car and car.engine are both created here
car.dims.length_m = 4.76;
```

### `struct()` constructor

Build a struct from key–value pairs:

```matlab
s = struct('x', 1, 'y', 2)     % two fields
p = struct('name', 'Alice', 'score', 98.5)
e = struct()                    % empty struct (zero fields)
```

Arguments must come in pairs (name, value). The name must be a string
(`'single-quoted'` or `"double-quoted"`).

---

## Reading fields

Use the same `.` notation:

```matlab
pt.x          % 3
pt.y          % 4

car.engine.hp          % 190
car.dims.length_m      % 4.76
```

Chaining works to any depth. Accessing a field that does not exist is an error.

---

## Dynamic field access

Use `s.(expr)` to read or write a field whose name is computed at runtime.
The expression inside `.(...)` must evaluate to a string:

```matlab
fname = 'x';
s.x = 10;
s.(fname)           % 10 — equivalent to s.x

s.(fname) = 99;     % write: equivalent to s.x = 99
s.x                 % 99
```

This is especially useful when iterating over a list of field names:

```matlab
stats.min  = -3.14;
stats.max  =  9.81;
stats.mean =  2.71;

fields = {'min', 'max', 'mean'};
for k = 1:numel(fields)
  fprintf('  %s = %g\n', fields{k}, stats.(fields{k}))
end
```

Or when building a struct from parallel name/value arrays:

```matlab
keys   = {'x', 'y', 'z'};
values = {10,  20,  30};
pt = struct();
for k = 1:numel(keys)
  pt.(keys{k}) = values{k};
end
pt.y    % 20
```

An inline string literal also works: `s.('fieldname')`.

The field expression must evaluate to a string; passing a number produces an
error: `"Dynamic field name must be a string"`.

---

## Built-in utilities

| Function            | Description                                                   |
|---------------------|---------------------------------------------------------------|
| `fieldnames(s)`     | Cell array of field names in insertion order                  |
| `isfield(s, 'x')`  | `1` if field `'x'` exists, else `0`                          |
| `rmfield(s, 'x')`  | Copy of `s` with field `'x'` removed; error if absent        |
| `isstruct(v)`       | `1` if `v` is a struct, else `0`                             |

```matlab
s.a = 1;  s.b = 2;  s.c = 3;

fn = fieldnames(s)       % {'a'; 'b'; 'c'}
fn{1}                    % a
numel(fn)                % 3

isfield(s, 'b')          % 1
isfield(s, 'z')          % 0

s2 = rmfield(s, 'b');
fieldnames(s2)           % {'a'; 'c'}

isstruct(s)              % 1
isstruct(42)             % 0
```

---

## Display

```
s =

  scalar structure containing the fields:

    x: 3
    y: 4
    engine: [1×1 struct]
    data: [1×100 double]
```

Nested structs and non-scalar values are shown **inline** as
`[1×1 struct]`, `[M×N double]`, or `{1×N cell}`. Access the field directly
to see its full contents.

---

## Structs in functions

Pass and return structs like any other value:

```matlab
function d = distance(pt)
  d = sqrt(pt.x^2 + pt.y^2 + pt.z^2);
end

p = struct('x', 1, 'y', 2, 'z', 2);
distance(p)    % 3
```

Build structs inside functions the same way:

```matlab
function v = make_vec3(x, y, z)
  v.x = x;  v.y = y;  v.z = z;
end

u = make_vec3(1, 0, 0);
u.x    % 1
```

---

## Nested structs

```matlab
config.server.host = 'localhost';
config.server.port = 8080;
config.db.name     = 'prod';
config.db.timeout  = 30;

config.server.host    % localhost
config.db.timeout     % 30
```

`fieldnames` returns only the **top-level** fields:

```matlab
fieldnames(config)    % {'server'; 'db'}
```

---

## Workspace

Structs are **not** saved by `ws` / `save` — the same policy as matrices, complex
values, and cell arrays.

`who` displays structs as:

```
s = [1×1 struct]
```

---

---

## Struct arrays

A **struct array** is a 1-D array of structs that all share the same field
schema. Use indexed assignment to create and grow the array:

```matlab
pts(1).x = 1;  pts(1).y = 0;
pts(2).x = 3;  pts(2).y = 4;
pts(3).x = 0;  pts(3).y = 5;

numel(pts)     % 3
isstruct(pts)  % 1
pts(2).x       % 3
```

Access an element by index — it returns a scalar struct:

```matlab
p = pts(1);
p.x      % 1
p.y      % 0

pts(3).y   % 5  (chained access also works)
```

### Field collection

Applying `.field` to the array (without an index) collects that field across
all elements:

```matlab
xs = pts.x;   % [1 3 0]   — 1×3 row vector (all scalars)
ys = pts.y;   % [0 4 5]

dists = (xs .^ 2 + ys .^ 2) .^ 0.5;   % [1 5 5]
```

If the field holds non-scalar values, the result is a cell array instead of
a matrix.

### Building in a loop

Struct arrays grow automatically:

```matlab
for k = 1:5
  data(k).value = k * k;
  data(k).label = num2str(k);
end

vals = data.value;   % [1 4 9 16 25]
sum(vals)            % 55
```

### String fields → cell array

When a collected field holds strings, the result is a cell array:

```matlab
roster(1).name = 'Alice';  roster(1).score = 92;
roster(2).name = 'Bob';    roster(2).score = 78;

names  = roster.name;    % {'Alice', 'Bob'}  — cell array
scores = roster.score;   % [92 78]           — matrix

names{1}      % Alice
mean(scores)  % 85
```

### Built-in utilities on struct arrays

`fieldnames`, `isfield`, `rmfield`, `numel`, `size`, `length`, and `isstruct`
all work on struct arrays the same way they do on scalar structs.

```matlab
fn = fieldnames(pts);
fn{1}               % x
numel(fn)           % 2
isfield(pts, 'x')   % 1
isfield(pts, 'z')   % 0
```

### Display

```
pts =

  1×3 struct array with fields:
    x
    y
```

A single-element struct array (`[1×1 struct]`) displays its full field values
like a scalar struct.

---

## See also

- `help structs` — in-REPL reference
- `help cells` — cell arrays, `varargin`/`varargout`
- `ccalc examples/structs.calc` — annotated scalar struct example
- `ccalc examples/struct_arrays.calc` — annotated struct array example
- `ccalc examples/dyn_field_demo.m` — dynamic field access examples
