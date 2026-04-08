% sequences.calc — Phase 5 (range, linspace) + Phase 6 (indexing) demo
%
% Topic: polynomial evaluation on a uniform grid, with slicing
%        and a finite-difference approximation of the derivative.
%
% Polynomial: f(x) = x^2 - x - 2  (roots at x = -1 and x = 2)
%
% Usage: ccalc sequences.calc

% --- 1. Integer sequences ---

fprintf('=== 1. Integer sequences ===\n')

n = 1:5;
fprintf('1:5 = ')
disp(n)

% Every other value
odd = 1:2:9;
fprintf('1:2:9 = ')
disp(odd)

% Countdown
countdown = 5:-1:1;
fprintf('5:-1:1 = ')
disp(countdown)

% Ranges inside brackets: build a row vector mixing scalars and ranges
v = [0, 1:4, 10];
fprintf('[0, 1:4, 10] = ')
disp(v)

% Powers of 2 via element-wise operator on a range
pow2 = 2 .^ (0:7);
fprintf('2.^(0:7) = ')
disp(pow2)

% --- 2. Uniform grid with linspace ---

fprintf('\n=== 2. Uniform grid ===\n')

% 9 evenly spaced points from -2 to 2 (step = 0.5)
x = linspace(-2, 2, 9);
fprintf('x = linspace(-2, 2, 9):\n')
disp(x)

% Evaluate f(x) = x^2 - x - 2 on every grid point
y = x .^ 2 - x - 2;
fprintf('f(x) = x.^2 - x - 2:\n')
disp(y)

% --- 3. Verify roots by indexing ---

fprintf('\n=== 3. Verify roots ===\n')

% x(3) = -1 and x(9) = 2 are the roots of f
fprintf('x(3) — should be -1:            %.2f\n', x(3))
fprintf('y(3) = f(-1) — should be 0:     %.2f\n', y(3))

fprintf('x(9) — should be 2:             %.2f\n', x(9))
fprintf('y(9) = f(2)  — should be 0:     %.2f\n', y(9))

% --- 4. Sub-vector slicing ---

fprintf('\n=== 4. Sub-vector slicing ===\n')

% First half (x <= 0)
fprintf('x(1:5) — first half [−2..0]: ')
disp(x(1:5))

% Second half (x >= 0)
fprintf('x(5:9) — second half [0..2]: ')
disp(x(5:9))

% Middle element (x = 0)
fprintf('x(5) — midpoint: %.2f\n', x(5))

% --- 5. Finite differences — discrete derivative ---

fprintf('\n=== 5. Finite differences ===\n')

% dy(i) = y(i+1) - y(i)  for i = 1..8  (8 differences from 9 points)
dy = y(2:9) - y(1:8);

step = x(2) - x(1);      % grid spacing = 0.5
dydx = dy / step;         % scaled differences ≈ f'(x) at midpoints

fprintf('Discrete derivative dydx (8 values):\n')
disp(dydx)

% f'(x) = 2x - 1 = 0 at x = 0.5 (minimum of f).
% The sign of dydx changes between element 5 (at midpoint x = 0.25, dydx = -0.5)
% and element 6 (at midpoint x = 0.75, dydx = +0.5), confirming x_min ≈ 0.5.
fprintf('dydx(5) — should be -0.5 (f decreasing at x=0.25): %.2f\n', dydx(5))
fprintf('dydx(6) — should be +0.5 (f increasing at x=0.75): %.2f\n', dydx(6))

% --- 6. Matrix slicing ---

fprintf('\n=== 6. Matrix slicing ===\n')

% Build a 4x4 matrix using ranges as rows
H = [1:4; 5:8; 9:12; 13:16];
fprintf('H:\n')
disp(H)

% Single element
fprintf('H(2, 3) — should be 7: %d\n', H(2, 3))

% Extract a full row
fprintf('H(2, :) — row 2: ')
disp(H(2, :))

% Extract a full column
fprintf('H(:, 3) — column 3:\n')
disp(H(:, 3))

% Top-left 2x2 submatrix
fprintf('H(1:2, 1:2):\n')
disp(H(1:2, 1:2))

% Bottom-right 2x2 submatrix
fprintf('H(3:4, 3:4):\n')
disp(H(3:4, 3:4))
