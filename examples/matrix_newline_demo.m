% matrix_newline_demo.m — Phase 33b: newline as matrix row separator
%
% In MATLAB/Octave a bare newline inside [...] is a row separator, identical to ';'.
% This demo shows the feature working across common patterns.
%
% Usage: ccalc examples/matrix_newline_demo/matrix_newline_demo.m

fprintf('=== Multi-line matrix literals ===\n\n')

% ── 2×3 matrix ────────────────────────────────────────────────────────────────
% Identical to A = [1 2 3; 4 5 6]
A = [1 2 3
     4 5 6]

fprintf('A is %dx%d\n\n', size(A,1), size(A,2))

% ── column vector ─────────────────────────────────────────────────────────────
% Identical to v = [10; 20; 30; 40]
v = [10
     20
     30
     40]

fprintf('v is %dx%d\n\n', size(v,1), size(v,2))

% ── expressions on each row ───────────────────────────────────────────────────
x = pi/4;
trig = [sin(x)
        cos(x)
        tan(x)]

fprintf('trig is %dx%d\n\n', size(trig,1), size(trig,2))

% ── commas and spaces on the same row ─────────────────────────────────────────
% Mixing comma-separated elements with newline row breaks
B = [1, 2, 3
     4, 5, 6
     7, 8, 9]

fprintf('B is %dx%d, trace = %g\n\n', size(B,1), size(B,2), trace(B))

% ── inline comment on a row ───────────────────────────────────────────────────
% Comments after % are stripped before the newline is processed as a separator.
C = [100 200  % first row
     300 400] % second row

fprintf('C(2,1) = %g  (expected 300)\n\n', C(2,1))

% ── line continuation suppresses the newline ──────────────────────────────────
% '...' joins the next line into the same row (no row break).
D = [1 2 ...
     3 4]

fprintf('D is %dx%d  (expected 1x4)\n\n', size(D,1), size(D,2))

% ── equivalence check ─────────────────────────────────────────────────────────
E_nl = [1 2
        3 4];
E_sc = [1 2; 3 4];
if E_nl == E_sc
  fprintf('Newline and semicolon forms produce identical matrices.\n')
end
