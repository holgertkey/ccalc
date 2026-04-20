# Phase 15 — Indexed Assignment

**Version:** 0.21.0  
**Status:** Complete

## Overview

Phase 15 adds in-place modification of matrix elements: the write counterpart
to Phase 6's read-only indexing. It unlocks the full MATLAB/Octave programming
model — building vectors in loops, updating submatrices, and applying boolean
masks to filter or clamp data.

## New syntax

```
name(index)       = rhs
name(i, j)        = rhs
name(range)       = rhs
name(:)           = rhs
name(mask)        = rhs
```

Parsed as `Stmt::IndexSet { name, indices, value }`, detected at parse time by
`try_split_index_assign` — the same string-level lookahead strategy used for
`FieldSet` (Phase 13) and `CellSet` (Phase 12.5).

## 15a — Scalar and slice assignment

The right-hand side can be a scalar (broadcast to all selected positions) or a
vector/matrix matching the selection size.

```matlab
v = zeros(1, 6);

v(3) = 42;               % single element
v(1:2) = [10, 20];       % slice from a vector
v(4:6) = 99;             % scalar broadcast
v(:) = 0;                % reset all elements

A = zeros(4);
A(2, 3) = 7;             % 2-D element
A(:, 1) = [1; 2; 3; 4]; % full column
A(1, :) = [10 20 30 40]; % full row
A(2:3, 2:3) = eye(2);    % submatrix
```

## 15b — Growing vectors

Assigning to an index beyond the current length extends the storage and fills
gaps with zeros. `end+1` is the idiomatic append:

```matlab
squares = [];
for k = 1:8
  squares(end+1) = k^2;
end
% [1 4 9 16 25 36 49 64]

v = [1, 2, 3];
v(7) = 99;   % → [1 2 3 0 0 0 99]
```

A variable that doesn't yet exist is auto-created as a 1×N row vector.

## 15c — Cell element assignment

Cell array grow via `c{end+1} = val` was already supported as `Stmt::CellSet`
from Phase 12.5. Phase 15 ensures `end` is correctly injected in the write
path so the idiom works reliably.

## 15d — Logical (boolean mask) indexing

A 0/1 vector or matrix whose element count equals the dimension size is
interpreted as a boolean mask rather than an index list. This allows
conditional reads and writes with a single expression.

```matlab
temps = [18, 22, 35, 12, 29, 41, 8, 33];

% Read with mask
hot = temps(temps >= 30);   % → [35 41 33]

% Write with mask
temps(temps >= 30) = 30;    % cap at 30

% Separate mask variable
noise = signal < 0;
signal(noise) = 0;          % half-wave rectifier

% 2-D mask (elements selected in column-major order)
M = [1 2 3; 4 5 6; 7 8 9];
M(M > 5) = 0;
```

## Bug fix: `zeros(n)` / `ones(n)`

Single-argument forms now correctly create an n×n matrix (previously required
the two-argument form).

## Tests added

14 regression tests added in `parser_tests.rs`:
- Scalar/slice/broadcast assignment (1-D and 2-D)
- Row/column/submatrix assignment
- Vector growth with `end+1` and out-of-range index
- Logical mask read and write (1-D and 2-D)
- `zeros(n)` / `ones(n)` single-argument form
- `c{end+1}` cell grow

## Example file

```bash
ccalc examples/indexed_assignment.calc
```

Covers all sub-phases: element and slice assignment, 2-D matrix assignment,
vector growth with `end+1`, cell array growth, and logical mask indexing with
a half-wave rectifier and practical filter examples.
