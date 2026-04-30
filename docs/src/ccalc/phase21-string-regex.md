# Phase 21 — String Completions and Regex

**Version:** v0.26.0  
**Prerequisite:** Phase 9 (string types), Phase 12.5 (cell arrays)

---

## 21a — String predicates and joining

### `contains`

```
contains(s, pat)                        % substring check
contains(s, pat, 'IgnoreCase', true)    % case-insensitive
```

Returns `1` if `pat` is found anywhere in `s`, `0` otherwise.

### `startsWith` / `endsWith`

```
startsWith(s, pat)   % 1 if s begins with pat
endsWith(s, pat)     % 1 if s ends with pat
```

### `strjoin`

```
strjoin(c)         % join with space (default)
strjoin(c, delim)  % join with explicit delimiter
```

`c` must be a cell array of char arrays or string objects.
Returns a char array (`Value::Str`).

---

## 21b — Regular expressions

Feature-gated behind `--features regex`. Without the feature, calling
any of these functions returns an error; their names always appear in
tab completion.

### `regexp`

```
regexp(s, pat)           % 1-based start index of first match, or []
regexp(s, pat, 'match')  % cell array of all matched substrings
```

### `regexpi`

Case-insensitive variants; same signatures as `regexp`.

### `regexprep`

```
regexprep(s, pat, rep)   % replace all matches with literal string rep
```

The replacement string is always treated as a literal — `$1`, `${name}`,
etc. are **not** expanded as capture group references.

---

## Implementation notes

- 21a functions are in `call_builtin` with no feature gate.
- 21b dispatches through `regexp_impl` / `regexprep_impl` helpers gated
  with `#[cfg(feature = "regex")]` / `#[cfg(not(feature = "regex"))]`.
- The `regex` crate (NFA engine) is an optional dependency; no risk of
  catastrophic backtracking.
- `regexp` byte-to-char offset conversion: `s[..m.start()].chars().count() + 1`.
