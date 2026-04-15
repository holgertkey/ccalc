# Structs

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

  struct with fields:

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

## See also

- `help structs` — in-REPL reference
- `help cells` — cell arrays, `varargin`/`varargout`
- `ccalc examples/structs.calc` — annotated 9-section example
