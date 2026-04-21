# Phase 16 — Package Namespaces

**Version:** v0.21.0+011  
**Status:** Complete

## Overview

Packages are directories whose name starts with `+` (e.g., `+utils`, `+geom`).
Functions inside are invisible at the top level; callers use the package prefix:

```matlab
utils.clamp(x, 0, 10)
geom.circle_area(r)
```

This mirrors MATLAB's package system and eliminates function-name collisions
across libraries.

## Implementation

### New AST node: `Expr::DotCall`

```rust
DotCall(Vec<String>, Vec<Expr>)
```

`segments` holds the dot-separated name components, e.g. `["utils", "clamp"]`.
Arguments follow as a normal expression list.

### Parser change

The postfix loop in `parse_primary` (parser.rs) now handles `Token::LParen`
after a `FieldGet`/`Var` chain. The new `field_chain_segments(e: &Expr)`
helper extracts the segment list from a pure `Var`/`FieldGet` chain; if the
result has two or more segments, a `DotCall` node is produced.

```
a.b(args)      → DotCall(["a", "b"],    [args])
a.b.c(args)    → DotCall(["a", "b", "c"], [args])
```

If the chain contains any non-`Var`/`FieldGet` node (e.g. a `Call`), the
`LParen` is left in the token stream for the caller to handle.

### Evaluator

`Expr::DotCall` is evaluated in two branches:

1. **Struct field call** — if `segments[0]` is in the environment, the segment
   chain is followed as field accesses (`FieldGet` semantics) and the resulting
   value is called with the evaluated arguments. Supports `Lambda` and `Function`
   field values.

2. **Package call** — if `segments[0]` is not in the environment, the qualified
   name (`"utils.clamp"`) is looked up in `AUTOLOAD_CACHE`. On a cache miss,
   `try_autoload` is called with the qualified name, which delegates to the
   new `try_autoload_pkg`.

### `try_autoload_pkg` (exec.rs)

Splits the qualified name into package segments and a function name, builds
the relative path `+pkg1/+pkg2/.../func`, and searches:

1. `SCRIPT_DIR_STACK` entries (calling script's directory)
2. CWD (`.`)
3. `SESSION_PATH` entries

On success, the function is loaded from the `.calc` (or `.m`) file and cached
in `AUTOLOAD_CACHE` under the qualified name (e.g. `"utils.clamp"`).

## Directory structure

```
+utils/
  clamp.calc       % function y = clamp(x, lo, hi)
  lerp.calc        % function y = lerp(a, b, t)

+geom/
  circle_area.calc % function a = circle_area(r)
  +solid/
    sphere_vol.calc % function v = sphere_vol(r)  → geom.solid.sphere_vol(r)
```

## Example

```bash
ccalc examples/scoping/scoping.calc
```

Section 8 of the scoping example demonstrates:
- `utils.clamp` and `utils.lerp` from `+utils/`
- `geom.circle_area` and `geom.rect_area` from `+geom/`
- Packages composed in expressions: `utils.clamp(utils.lerp(-10, 20, 0.5), 0, 10)`

## Tests

`cargo test` — all 667 tests pass.
