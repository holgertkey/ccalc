# containers.Map

`containers.Map` is a **string-keyed associative array** — a lookup table that
maps string keys to values of any type. It is the ccalc equivalent of Python's
`dict` or JavaScript's `Map`.

---

## Creating a map

```matlab
% Empty map
m = containers.Map();

% From two cell arrays — keys cell + values cell (must be equal length)
prices = containers.Map({'apple', 'banana', 'cherry'}, {1.5, 0.75, 2.0});
```

All keys must be strings (char arrays or string objects).  
Values can be any type: scalar, matrix, string, cell, struct, etc.

---

## Reading values

Use parenthesis indexing with a string key:

```matlab
prices('apple')     % → 1.5
prices('banana')    % → 0.75
```

Accessing an absent key is an error:

```matlab
prices('mango')     % error: Map key 'mango' not found
```

---

## Writing values

```matlab
prices('date') = 3.5;      % insert new key
prices('banana') = 0.99;   % update existing key
```

---

## Count property

```matlab
prices.Count    % → number of entries (read-only)
```

---

## Built-in functions

| Function | Description |
|---|---|
| `isKey(m, 'key')` | `1` if key is present, `0` otherwise |
| `keys(m)` | Cell array of all keys, **sorted alphabetically** |
| `values(m)` | Cell array of values in the same sorted-key order |
| `remove(m, 'key')` | Remove a key in-place (no assignment needed) |

```matlab
m = containers.Map({'c', 'a', 'b'}, {3, 1, 2});

isKey(m, 'a')   % → 1
isKey(m, 'z')   % → 0

k = keys(m)     % → {'a', 'b', 'c'}   (sorted)
v = values(m)   % → {1, 2, 3}         (matching key order)

remove(m, 'b');
m.Count         % → 2
```

---

## Iterating over a map

```matlab
m = containers.Map({'x', 'y', 'z'}, {10, 20, 30});
k = keys(m);
for i = 1:m.Count
  fprintf('%s = %g\n', k{i}, m(k{i}));
end
```

---

## Display

```
m =

  Map with 3 entries:

    'apple'  → 1.5
    'banana' → 0.75
    'cherry' → 2
```

---

## Notes

- **String keys only.** Numeric-key maps are not supported.
- **Value semantics.** Assigning `m2 = m` creates a copy (unlike MATLAB handle
  semantics). Mutations to `m2` do not affect `m`.
- **Maps are not persisted** by `ws`/`save` — same policy as matrices and cells.
- `remove(m, k)` mutates `m` in-place without an assignment statement, matching
  MATLAB handle-class behaviour as closely as possible under value semantics.

---

## See also

[Cell Arrays](cell-arrays.md) · [Structs](structs.md)
