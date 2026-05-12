% complex_matrix_ext.m — Phase 27.5 feature demo
%
% Covers: indexed assignment into ComplexMatrix (auto-upcast),
%         block matrix concatenation with ComplexMatrix sub-blocks,
%         complex eigenvalues from non-symmetric real matrices,
%         and the extended built-ins trace/diag/sum/prod/mean on ComplexMatrix.
%
% Usage: ccalc complex_matrix_ext.m

% --- 1. Indexed assignment: auto-upcast ---
%
% Assigning a complex value into a real matrix automatically promotes it
% to ComplexMatrix.  All existing real entries are preserved as x+0i.

fprintf('=== 1. Indexed assignment / auto-upcast ===\n')

A = zeros(3, 3);
A(2, 2) = 1 + 2i;         % real matrix → ComplexMatrix on first complex write
A(1, :) = [3-1i, 0, 2i];  % range assignment with a complex row vector
fprintf('A after assignment:\n')
disp(A)

% Works the same way with linear indexing on a vector
v = [10, 20, 30, 40];
v(3) = 5 + 7i;
fprintf('v after v(3) = 5+7i:\n')
disp(v)

% Write back into a ComplexMatrix LHS with a real scalar — it stays ComplexMatrix
A(2, 2) = 9;
fprintf('A(2,2) reset to 9 (stays ComplexMatrix):\n')
disp(A)

% --- 2. Block matrix concatenation ---
%
% ComplexMatrix blocks can be freely mixed with real Matrix blocks in
% horizontal [A, B] or vertical [A; B] concatenation.

fprintf('\n=== 2. Block matrix concatenation ===\n')

Z = [1+2i, 3-4i; 5, 6+1i];
O = ones(2, 2);

fprintf('Z (2x2 ComplexMatrix):\n')
disp(Z)

fprintf('[Z, O]  (horizontal):\n')
disp([Z, O])

fprintf('[Z; O]  (vertical):\n')
disp([Z; O])

fprintf('[Z, Z; O, O]  (2x2 block):\n')
disp([Z, Z; O, O])

% --- 3. trace, diag, sum, prod, mean ---
%
% All five now work element-wise or column-wise on ComplexMatrix,
% following the same conventions as for real Matrix.

fprintf('\n=== 3. ComplexMatrix reductions ===\n')

M = [1+2i, 3+4i; 5+6i, 7+8i];
fprintf('M:\n')
disp(M)

t = trace(M);
fprintf('trace(M) = %.0f + %.0fi    (sum of diagonal)\n', real(t), imag(t))

d = diag(M);
fprintf('diag(M)  = main diagonal column vector:\n')
disp(d)

fprintf('diag(d)  = diagonal matrix built from vector:\n')
disp(diag(d))

s = sum(M);
fprintf('sum(M)   = column sums (1x2 ComplexMatrix):\n')
disp(s)

p = prod(M);
fprintf('prod(M)  = column products:\n')
disp(p)

mu = mean(M);
fprintf('mean(M)  = column means:\n')
disp(mu)

% Reduction on a vector collapses to a single complex scalar
sv = sum([1+2i, 3+4i, 5+6i]);
fprintf('sum([1+2i, 3+4i, 5+6i]) = %.0f + %.0fi\n\n', real(sv), imag(sv))

% --- 4. Complex eigenvalues from a real matrix ---
%
% A real non-symmetric matrix can have complex conjugate eigenvalue pairs.
% eig() now detects 2x2 blocks in the quasi-triangular Schur form and
% returns a ComplexMatrix column vector of eigenvalues.
%
% Stability check: all eigenvalues must have negative real parts for a
% continuous-time system to be stable.

fprintf('=== 4. Complex eigenvalues (stability analysis) ===\n')

% Rotation matrix — eigenvalues are +i and -i (pure imaginary → neutrally stable).
% The QR iteration produces tiny numerical noise in the real part (~1e-13); round to 12
% decimal places to show the mathematically exact result.
Rot = [0, -1; 1, 0];
eRot = eig(Rot);
eRot = round(real(eRot) * 1e12) / 1e12 + i * imag(eRot);
fprintf('eig([0,-1;1,0])  (rotation, eigenvalues = ±i):\n')
disp(eRot)

% Damped oscillator: A = [0,1; -omega^2, -2*zeta*omega]
% omega = 2 (natural frequency), zeta = 0.3 (damping ratio)
omega = 2;
zeta  = 0.3;
Aosc  = [0, 1; -(omega^2), -2*zeta*omega];
eOsc  = eig(Aosc);
fprintf('Damped oscillator (omega=2, zeta=0.3) eigenvalues:\n')
disp(eOsc)
fprintf('  Real parts  (< 0 → stable):  ')
disp(real(eOsc)')
fprintf('  Imag parts  (damped frequency): ')
disp(round(imag(eOsc)' * 1e4) / 1e4)

% Unstable system: trace > 0 forces at least one eigenvalue into the right half-plane.
% [0.5, 1; -1, 0.3] has trace=0.8 > 0, so Re(λ) = 0.4 for both eigenvalues.
Aunst = [0.5, 1; -1, 0.3];
eUnst = eig(Aunst);
fprintf('Unstable system eigenvalues:\n')
disp(eUnst)
stable = all(real(eUnst) < 0);
fprintf('  Stable: %d\n\n', stable)

% --- 5. Practical: Companion matrix of a polynomial ---
%
% The eigenvalues of the companion matrix of p(x) = x^4 + 2x^3 + 4x^2 + 3x + 1
% are the roots of the polynomial.
%
% Companion matrix (column of -coefficients / leading term):
%   [0  0  0  -c0]
%   [1  0  0  -c1]
%   [0  1  0  -c2]
%   [0  0  1  -c3]
% where p(x) = x^4 + c3*x^3 + c2*x^2 + c1*x + c0

fprintf('=== 5. Roots via companion matrix ===\n')

% p(x) = x^4 + 2x^3 + 4x^2 + 3x + 1
c  = [1, 3, 4, 2];   % coefficients [c0, c1, c2, c3] (excluding leading 1)
n  = length(c);
C  = zeros(n, n);
for k = 1:n-1
    C(k+1, k) = 1;
end
C(:, n) = -c';
fprintf('Companion matrix C:\n')
disp(C)

roots_p = eig(C);
fprintf('Roots of x^4 + 2x^3 + 4x^2 + 3x + 1:\n')
disp(roots_p)

% Verify: sum of roots = -c3 (Vieta's formula)
sr = sum(roots_p);
fprintf('Sum of roots = %.4f + %.4fi  (should equal -2)\n', real(sr), imag(sr))
