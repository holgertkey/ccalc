% Linear algebra — Phase 18 feature demo
%
% Covers: qr, lu, chol, svd, eig, rank, null, orth, cond, pinv, norm
%
% Usage: ccalc linear_algebra.calc

% --- 1. QR decomposition ---
%
% A = Q * R  where Q is full orthogonal (m×m), R is upper triangular (m×n).
% "Thin" (economy) Q has only the first n columns; thin R is n×n square.

A = [1 2; 3 4; 5 6];
[Q, R] = qr(A);
fprintf('A (3x2):\n')
disp(A)
fprintf('Q (full orthogonal, 3x3):\n')
disp(Q)
fprintf('R (upper triangular, 3x2):\n')
disp(R)
fprintf("Q'*Q — identity (3x3):\n")
disp(Q' * Q)
fprintf('||Q*R - A|| (Frobenius): %.2e\n', norm(Q * R - A, 'fro'))

% --- 2. Least-squares via thin QR: fit y = a + b*x to noisy data ---
%
% For overdetermined Af*c ≈ y, extract the thin factors:
%   Q1 = Q(:, 1:n)  (first n columns)
%   R1 = R(1:n, :)  (first n rows — square upper triangular)
% Then c = R1 \ (Q1' * y)

x_data = [1; 2; 3; 4; 5];
y_data = [2.1; 3.9; 6.2; 7.8; 10.1];
Af = [ones(5, 1), x_data];
[Qf, Rf] = qr(Af);
Q1 = Qf(:, 1:2);
R1 = Rf(1:2, :);
c = R1 \ (Q1' * y_data);
fprintf('\nLeast-squares line fit  y = a + b*x  through 5 noisy points:\n')
fprintf('  intercept a = %.4f\n', c(1))
fprintf('  slope     b = %.4f\n', c(2))
fprintf('  residual ||Af*c - y||: %.4f\n', norm(Af * c - y_data))

% --- 3. LU decomposition ---
%
% PA = LU  with partial pivoting: P permutation, L unit lower triangular,
% U upper triangular.  Used internally by backslash (\).

B = [2, 1, -1; -3, -1, 2; -2, 1, 2];
[L, U, P] = lu(B);
fprintf('\nB:\n')
disp(B)
fprintf('L (unit lower triangular):\n')
disp(L)
fprintf('U (upper triangular):\n')
disp(U)
fprintf('||P*B - L*U||: %.2e\n', norm(P * B - L * U, 'fro'))

b_rhs = [8; -11; -3];
x_lu = B \ b_rhs;
fprintf('Solution to B*x = b via \\: ')
disp(x_lu')
fprintf('Residual: %.2e\n', norm(B * x_lu - b_rhs))

% --- 4. Cholesky decomposition ---
%
% For symmetric positive-definite A:  A = R' * R  (R upper triangular).
% Twice as fast as LU; an error is returned if A is not positive definite.

Spd = [4 2 2; 2 5 3; 2 3 6];
Rc = chol(Spd);
fprintf('\nSPD matrix A:\n')
disp(Spd)
fprintf('R = chol(A):\n')
disp(Rc)
fprintf("||R'*R - A||: %.2e\n", norm(Rc' * Rc - Spd, 'fro'))
fprintf('Condition number: %.4f\n', cond(Spd))

% --- 5. SVD — singular value decomposition ---
%
% A = U * S * V'  where U (m×m) and V (n×n) are orthogonal, S (m×n) diagonal.
% Reveals numerical rank, norms, and principal directions.

C = [1 2 3; 4 5 6; 7 8 9];
fprintf('\nC (3x3, rank 2 — rows are linearly dependent):\n')
disp(C)

s = svd(C);
fprintf('Singular values (column vector): ')
disp(s)
fprintf('rank(C)          = %d\n', rank(C))
fprintf('2-norm (max sv)  = %.4f\n', norm(C))
fprintf('Frobenius norm   = %.4f\n', norm(C, 'fro'))

[U, S, V] = svd(C);
fprintf('\nU (left singular vectors, 3x3):\n')
disp(U)
fprintf('S (diagonal matrix, 3x3):\n')
disp(S)
fprintf('V (right singular vectors, 3x3):\n')
disp(V)
fprintf("||U*S*V' - C||: %.2e\n", norm(U * S * V' - C, 'fro'))

% Economy SVD on a tall matrix (more rows than columns):
%   full U is m×m; economy U is m×n — saves memory for large m.
At = [1 2; 3 4; 5 6; 7 8];
[Ue, Se, Ve] = svd(At, 'econ');
fprintf('\nEconomy SVD of 4x2 matrix At:\n')
fprintf('Ue (4x2):\n')
disp(Ue)
fprintf('Se (2x2):\n')
disp(Se)
fprintf('Ve (2x2):\n')
disp(Ve)
fprintf("||Ue*Se*Ve' - At||: %.2e\n", norm(Ue * Se * Ve' - At, 'fro'))

% Rank-1 approximation of C: retain only the largest singular triplet
C1 = S(1, 1) * (U(:, 1) * V(:, 1)');
fprintf('Rank-1 approximation of C:\n')
disp(C1)
fprintf('||C - C1|| / ||C||: %.4f  (relative error)\n', ...
    norm(C - C1, 'fro') / norm(C, 'fro'))

% --- 6. Eigendecomposition ---
%
% For symmetric A:  A * V = V * D  where D is diagonal (eigenvalues),
% V is orthogonal (eigenvectors in columns).

Sym = [4 1 0; 1 3 1; 0 1 2];
fprintf('\nSymmetric matrix S:\n')
disp(Sym)

d = eig(Sym);
fprintf('Eigenvalues (column vector): ')
disp(d)

[EV, EW] = eig(Sym);
fprintf('Eigenvectors V (columns):\n')
disp(EV)
fprintf('Eigenvalue matrix D (diagonal):\n')
disp(EW)

% Verify  S*v_k = lambda_k * v_k  for each eigenpair — use EW(k,k) for k-th eigenvalue
max_res = 0;
for k = 1:size(EV, 2)
    r = norm(Sym * EV(:, k) - EW(k, k) * EV(:, k));
    if r > max_res
        max_res = r;
    end
end
fprintf('Max ||S*v_k - lambda_k*v_k||: %.2e\n', max_res)

% --- 7. Matrix properties ---

fprintf('\n--- Matrix properties ---\n')

D = [1 2 3; 4 5 6; 7 8 9];
fprintf('\nD (same rank-2 matrix as C above):\n')
disp(D)

% Rank
fprintf('rank(D): %d\n', rank(D))

% Null space — a vector x such that D*x = 0
N = null(D);
fprintf('null(D) — basis for the null space:\n')
disp(N)
fprintf('||D * null(D)||: %.2e  (should be near zero)\n', norm(D * N))

% Orthonormal column-space basis
On = orth(D);
fprintf('orth(D) — orthonormal basis for column space:\n')
disp(On)
fprintf("||orth(D)'*orth(D) - I||: %.2e\n", norm(On' * On - eye(size(On, 2)), 'fro'))

% Condition number
fprintf('\ncond(eye(3))          = %.2f  (perfectly conditioned)\n', cond(eye(3)))
Ill = [1 1; 1 1.0001];
fprintf('cond([1 1; 1 1.0001]) = %.4e  (nearly singular)\n', cond(Ill))

% Pseudoinverse (Moore-Penrose)
fprintf('\npinv(D) — pseudoinverse of rank-2 matrix:\n')
Dp = pinv(D);
disp(Dp)
fprintf('||D * pinv(D) * D - D||: %.2e  (fundamental property)\n', ...
    norm(D * Dp * D - D, 'fro'))
fprintf('rank(pinv(D)): %d  (same as rank(D))\n', rank(Dp))

% --- 8. Matrix norm summary ---

fprintf('\n--- Matrix norms ---\n')
M = [1 2; 3 4; 5 6];
fprintf('M:\n')
disp(M)
fprintf("norm(M)       2-norm  (largest singular value): %.4f\n", norm(M))
fprintf("norm(M,'fro') Frobenius (sqrt sum of squares):  %.4f\n", norm(M, 'fro'))
fprintf("norm(M,1)     max column sum:                   %.4f\n", norm(M, 1))
fprintf("norm(M,inf)   max row sum:                      %.4f\n", norm(M, inf))
