# Error Handling

ccalc provides MATLAB-compatible error handling so scripts can recover from runtime errors without crashing the session.

## Raising errors

```matlab
error('message')                   % plain message
error('expected %d, got %d', 2, n) % formatted (same as fprintf)
warning('result may be inaccurate') % prints to stderr, continues
```

## try / catch / end

```matlab
try
  result = risky_computation(x)
catch e
  fprintf('failed: %s\n', e.message)
  result = default_value
end
```

- If the `try` body succeeds, `catch` is skipped.
- `catch e` binds a struct with field `message` to the catch variable.
- Anonymous `catch` (no variable) silently handles the error.
- `try` with no `catch` silently swallows errors.

## Inline fallback: `try(expr, default)`

```matlab
n = try(str2num(s), 0)      % 0 if s is not a valid number
x = try(inv(A), eye(n))     % identity matrix if A is singular
```

The default is only evaluated if `expr` raises an error.

## Protected call: `pcall`

```matlab
[ok, val] = pcall(@func, arg1, arg2)
if ok
  % use val
else
  fprintf('error: %s\n', val)
end
```

Returns `[1, result]` on success and `[0, message]` on failure.

## Last error message

```matlab
lasterr()      % message from most recent error
lasterr('')    % clear
```

## "Did you mean?" hints

When a name is not found, ccalc compares it against all known variable names and
built-in function names using edit distance. If a close match exists (at most 2
edits away), it is shown as a suggestion:

```
>> sqrtt(4)
Error: Unknown function 'sqrtt'; did you mean 'sqrt'?

>> my_valu + 1
Error: Undefined variable 'my_valu'; did you mean 'my_value'?
```

No suggestion is shown when no close match exists.

## Source location in error messages

Errors inside block statements, function bodies, and scripts executed via
`run()`/`source()` include a `near line N` suffix pointing to the failing line:

```
Error: Undefined variable: 'v' near line 3
```

The line number is 1-based and relative to the immediately enclosing block or
function body, matching Octave's convention.

When an error propagates through nested blocks the **innermost** location is
kept — outer wrappers do not overwrite it. Inside a `catch` block, `e.message`
contains the original message without the `near line` suffix.

See [`help errors`](../ccalc/phase14-error-handling.md) for the full reference.
