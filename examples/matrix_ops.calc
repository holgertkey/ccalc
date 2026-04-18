% Matrix operations — Phase 4 + Phase 13.6 feature demo
%
% Covers: matrix multiply (*), transpose ('), element-wise (.* ./ .^),
%         built-ins: zeros, ones, eye, size, numel, trace, det, inv,
%         and the backslash operator \ (left division / linear solve)
%
% Usage: ccalc matrix_ops.calc

% --- 1. Rotation matrix (2D, 45 degrees) ---

theta = 45 * pi / 180;
R = [cos(theta), -sin(theta); sin(theta), cos(theta)];
fprintf('Rotation matrix R (45 deg):\n')
disp(R)

% For any rotation: det = 1, trace = 2*cos(theta)
fprintf('det(R) — must be 1:   %.2f\n', det(R))
fprintf('trace(R) = 2*cos(45): %.2f\n', trace(R))

% --- 2. Rotate a 2D point ---

p = [3; 0];
q = R * p;
fprintf('Original point p: ')
disp(p')
fprintf('Rotated point  q: ')
disp(q')

% Verify norm is preserved: p'*p == q'*q
fprintf('p''*p (norm squared): ')
disp(p' * p)
fprintf('q''*q (should match): ')
disp(q' * q)

% Inverse rotation: R' = R^-1 for orthogonal matrices
p_back = R' * q;
fprintf('R'' * q (back to p):\n')
disp(p_back)

% --- 3. Solving a 2x2 linear system: A*x = b ---
%
% Two equivalent approaches:
%   inv(A)*b  — forms the inverse explicitly (less stable, two operations)
%   A \ b     — left division: solves directly via Gaussian elimination (preferred)

A = [2 1; 5 7];
b = [11; 13];

fprintf('A:\n')
disp(A)
fprintf('b: ')
disp(b')

x1 = inv(A) * b;
fprintf('Solution via inv(A)*b: ')
disp(x1')

x2 = A \ b;
fprintf('Solution via A \\ b:    ')
disp(x2')

% Both give the same result; \ is numerically more stable and concise.
% Verify: residual A*x - b should be near zero
r = A * x2 - b;
fprintf('Residual ||A*x - b||:  %.2e\n', norm(r))

% --- 4. Element-wise operations on a column vector ---

v = [1; 2; 3; 4];
fprintf('v'' = ')
disp(v')
fprintf('v .^ 2 (each element squared):\n')
disp(v .^ 2)
fprintf('v .* v (same via .*):  ')
disp((v .* v)')
fprintf('v'' * v (dot product):  ')
disp(v' * v)

% Scale and normalize elements
w = v ./ 4;
fprintf('v ./ 4 = ')
disp(w')

% --- 5. Special matrices and properties ---

fprintf('zeros(2, 3):\n')
disp(zeros(2, 3))

fprintf('eye(3):\n')
disp(eye(3))

% 3x3 matrix with det = 1 (unimodular)
B = [1 2 3; 0 1 4; 5 6 0];
fprintf('B:\n')
disp(B)
fprintf('det(B):   %.2f\n', det(B))
fprintf('size(B): ')
disp(size(B))
fprintf('numel(B): %d\n', numel(B))

% Inverse and verification
Binv = inv(B);
fprintf('inv(B):\n')
disp(Binv)
fprintf('inv(B) * B — should be identity:\n')
disp(Binv * B)

% --- 6. Backslash: solving multiple right-hand sides at once ---
%
% A \ B where B has multiple columns solves each column independently.
% Equivalent to [A\b1, A\b2, ...] but in a single operation.

C = [4 1; 2 3];
B2 = [5 1; 10 0];        % two right-hand sides as columns

X = C \ B2;
fprintf('\nC \ B (two RHS at once):\n')
disp(X)
fprintf('Verify C * X - B (should be ~0):\n')
disp(C * X - B2)

% --- 7. Scalar left division ---
%
% a \ b = b / a  — useful shorthand when a is a scalar denominator

fprintf('Scalar: 4 \ 20 = %g\n', 4 \ 20)
fprintf('Scalar: 3 \ [6; 9; 12] = ')
disp((3 \ [6; 9; 12])')
