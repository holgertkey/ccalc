% set_operations.m — Phase 23 feature demo
%
% Covers: triu/tril, repmat, kron,
%         cross, dot,
%         intersect, union, setdiff, ismember,
%         sub2ind, ind2sub, repelem.
%
% Usage:
%   ccalc examples/set_operations.m
%   cargo run -- examples/set_operations.m

% ── 1. Triangular extraction ─────────────────────────────────────────────────

fprintf('=== 1. Triangular extraction ===\n')

A = [1 2 3; 4 5 6; 7 8 9];
fprintf('A =\n')
disp(A)

fprintf('triu(A) — upper triangular:\n')
disp(triu(A))

fprintf('triu(A, 1) — above main diagonal:\n')
disp(triu(A, 1))

fprintf('tril(A) — lower triangular:\n')
disp(tril(A))

fprintf('tril(A, -1) — below main diagonal:\n')
disp(tril(A, -1))

% ── 2. Tiling and Kronecker product ─────────────────────────────────────────

fprintf('=== 2. Tiling and Kronecker product ===\n')

B = [1 2; 3 4];
fprintf('repmat([1 2; 3 4], 2, 3) — 4x6 tile:\n')
disp(repmat(B, 2, 3))

% kron: identity scales each block independently
I2 = eye(2);
fprintf('kron(eye(2), [1 2; 3 4]) — 4x4 block-diagonal:\n')
disp(kron(I2, B))

% kron: all-ones matrix sums all blocks
fprintf('kron(ones(2), [1 2; 3 4]) — 4x4 with all blocks equal:\n')
disp(kron(ones(2), B))

% ── 3. Vector products ───────────────────────────────────────────────────────

fprintf('=== 3. Vector products ===\n')

% Cross product — right-hand rule for unit vectors
fprintf('cross([1 0 0], [0 1 0]):  ')
disp(cross([1 0 0], [0 1 0]))

fprintf('cross([1 2 3], [4 5 6]):  ')
disp(cross([1 2 3], [4 5 6]))

% cross of a vector with itself is zero
v = [3 1 4];
fprintf('cross(v, v) — should be zero:  ')
disp(cross(v, v))

% Dot product
fprintf('dot([1 2 3], [4 5 6]):  %g\n', dot([1 2 3], [4 5 6]))

% Verify orthogonality: cross product is perpendicular to both inputs
a = [1 2 3];
b = [4 5 6];
c = cross(a, b);
fprintf('dot(cross(a,b), a) = %g  (should be 0)\n', dot(c, a))
fprintf('dot(cross(a,b), b) = %g  (should be 0)\n', dot(c, b))

% ── 4. Set operations ────────────────────────────────────────────────────────

fprintf('\n=== 4. Set operations ===\n')

scores_a = [85 90 72 88 95 72];
scores_b = [90 78 95 65 88];

fprintf('scores_a = [85 90 72 88 95 72]\n')
fprintf('scores_b = [90 78 95 65 88]\n\n')

common = intersect(scores_a, scores_b);
fprintf('intersect — scores in both groups:\n')
disp(common)

all_scores = union(scores_a, scores_b);
fprintf('union — all unique scores:\n')
disp(all_scores)

only_a = setdiff(scores_a, scores_b);
fprintf('setdiff — scores only in group A:\n')
disp(only_a)

% ismember for a single query
target = 90;
fprintf('ismember(%g, scores_b) → %d\n', target, ismember(target, scores_b))

% ismember element-wise: which of scores_a appear in scores_b?
mask = ismember(scores_a, scores_b);
fprintf('ismember(scores_a, scores_b) → ')
disp(mask)
fprintf('  (1 = present in B, 0 = absent)\n')

% NaN is never a member — IEEE semantics
fprintf('ismember(nan, [nan 1 2]) → %d  (always 0)\n', ismember(nan, [nan 1 2]))

% ── 5. Index conversion ──────────────────────────────────────────────────────

fprintf('\n=== 5. Index conversion ===\n')

% 3×4 matrix indexing layout (column-major):
%   row 1: indices 1  4  7  10
%   row 2: indices 2  5  8  11
%   row 3: indices 3  6  9  12
sz = [3 4];

fprintf('sub2ind([3 4], 2, 3) → %g  (expected 8)\n', sub2ind(sz, 2, 3))
fprintf('sub2ind([3 4], 1, 4) → %g  (expected 10)\n', sub2ind(sz, 1, 4))

% Vectorised: convert several subscripts at once
rows_in = [1 2 3];
cols_in = [1 2 3];
linear  = sub2ind(sz, rows_in, cols_in);
fprintf('sub2ind([3 4], [1 2 3], [1 2 3]) → ')
disp(linear)

% Round-trip: linear → subscripts → linear
[r, c] = ind2sub(sz, 8);
fprintf('ind2sub([3 4], 8) → r=%g, c=%g  (expected 2, 3)\n', r, c)

[rs, cs] = ind2sub(sz, linear);
fprintf('ind2sub([3 4], [1 5 9]) → r=')
disp(rs)
fprintf('                          c=')
disp(cs)

% ── 6. Element repetition ────────────────────────────────────────────────────

fprintf('\n=== 6. Element repetition ===\n')

% Uniform repetition
fprintf('repelem([1 2 3], 3):\n')
disp(repelem([1 2 3], 3))

% Per-element repetition counts
counts = [3 1 2];
fprintf('repelem([10 20 30], [3 1 2]):\n')
disp(repelem([10 20 30], counts))

% 2-D repetition: each element becomes a 2×3 tile of itself
M = [1 2; 3 4];
fprintf('repelem([1 2; 3 4], 2, 3):\n')
disp(repelem(M, 2, 3))

% ── 7. Practical — voter overlap analysis ────────────────────────────────────

fprintf('\n=== 7. Practical — voter overlap analysis ===\n')

% Three precincts each report which polling booth IDs cast votes
precinct_1 = [101 105 112 118 125 130];
precinct_2 = [103 105 110 118 122 130 135];
precinct_3 = [100 105 112 120 130 140];

fprintf('Booths that voted in ALL three precincts:\n')
all3 = intersect(intersect(precinct_1, precinct_2), precinct_3);
disp(all3)

fprintf('Booths that voted in precincts 1 OR 2 but NOT 3:\n')
in_12    = union(precinct_1, precinct_2);
only_12  = setdiff(in_12, precinct_3);
disp(only_12)

fprintf('Was booth 118 active in precinct 3? → %d\n', ismember(118, precinct_3))
fprintf('Was booth 105 active in precinct 3? → %d\n', ismember(105, precinct_3))

% Use sub2ind to map a 4×8 grid of precincts to linear storage
grid_sz = [4 8];
p_row   = 2;
p_col   = 5;
storage_slot = sub2ind(grid_sz, p_row, p_col);
fprintf('\nPrecinct at grid position (%d,%d) → storage slot %g\n', p_row, p_col, storage_slot)

[back_r, back_c] = ind2sub(grid_sz, storage_slot);
fprintf('Recover position from slot %g → row=%g, col=%g\n', storage_slot, back_r, back_c)

fprintf('\nDone.\n')
