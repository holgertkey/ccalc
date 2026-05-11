% complex_matrix.m — Phase 27 feature demo
%
% Covers: complex matrix literals, arithmetic, transpose, element-wise
%         built-ins (real, imag, abs, conj, angle, isreal), indexing,
%         and a practical DFT-matrix example.
%
% Usage: ccalc complex_matrix.m

% --- 1. Complex matrix literals ---
%
% Any matrix literal where at least one element has a non-zero imaginary
% part becomes a ComplexMatrix.  Real elements are promoted automatically.

fprintf('=== 1. Complex matrix literals ===\n')

A = [1+2i, 3-4i; 5, 6+1i];
fprintf('A = [1+2i, 3-4i; 5, 6+1i]:\n')
disp(A)

v = [1+i, 2-i, 3];
fprintf('v = [1+i, 2-i, 3]:\n')
disp(v)

% Purely real matrix — stays a plain Matrix, not ComplexMatrix
R = [1, 2; 3, 4];
fprintf('isreal([1 2; 3 4]) = %d   (no imaginary part)\n', isreal(R))
fprintf('isreal(A)          = %d   (has imaginary parts)\n\n', isreal(A))

% --- 2. Arithmetic ---
%
% +, -, .*, ./, .^ work element-wise.
% * performs matrix multiplication.

fprintf('=== 2. Arithmetic ===\n')

B = [1-2i, 0+4i; 5, -1i];
fprintf('A + B:\n')
disp(A + B)

fprintf('A .* B  (element-wise multiply):\n')
disp(A .* B)

% Matrix multiply: row * column → scalar
P = [1+i, 1-i] * [1-i; 1+i];
fprintf('[1+i, 1-i] * [1-i; 1+i]  (dot product):\n')
disp(P)

% Scalar scaling
fprintf('2 * [1+i, 2+3i]:\n')
disp(2 * [1+i, 2+3i])

% Mixed real-matrix arithmetic
fprintf('[1+2i, 3+4i] + [10, 20]:\n')
disp([1+2i, 3+4i] + [10, 20])

% --- 3. Conjugate transpose and plain transpose ---
%
% A'   — conjugate transpose (Hermitian adjoint)
% A.'  — plain transpose (no conjugation)

fprintf('\n=== 3. Transpose ===\n')

M = [1+2i, 3+4i; 5+6i, 7+8i];
fprintf('M:\n')
disp(M)

Mh = M';
fprintf("M' (conjugate transpose):\n")
disp(Mh)

Mt = M.';
fprintf("M.' (plain transpose):\n")
disp(Mt)

% M * M' is Hermitian — its diagonal is real
MMh = M * Mh;
fprintf('real(M * M'')  (diagonal should be real):\n')
disp(real(MMh))

% --- 4. Element-wise built-ins ---

fprintf('\n=== 4. Element-wise built-ins ===\n')

Z = [3+4i, 0+2i; 1, -1-1i];
fprintf('Z:\n')
disp(Z)

fprintf('real(Z):\n')
disp(real(Z))

fprintf('imag(Z):\n')
disp(imag(Z))

fprintf('abs(Z)  (element-wise modulus):\n')
disp(round(abs(Z) * 1e4) / 1e4)

fprintf('conj(Z):\n')
disp(conj(Z))

fprintf('angle(Z)  (argument in radians):\n')
disp(round(angle(Z) * 1e4) / 1e4)

% --- 5. Indexing ---
%
% 1-based, column-major — same conventions as real matrices.

fprintf('\n=== 5. Indexing ===\n')

w = [10+1i, 20+2i, 30+3i, 40+4i];
fprintf('w = [10+i, 20+2i, 30+3i, 40+4i]\n')

fprintf('w(2)    =>  ')
disp(w(2))

fprintf('w(2:3)  =>  ')
disp(w(2:3))

% 2-D indexing
G = [1+1i, 2+2i; 3+3i, 4+4i];
fprintf('G(1,:) (first row):  ')
disp(G(1,:))

fprintf('G(:,2) (second column):\n')
disp(G(:,2))

% --- 6. size, numel, length, norm ---

fprintf('\n=== 6. Shape and norm ===\n')

C = [1+1i, 2-1i, 3; 4, 5+2i, 6-3i];
fprintf('C is %dx%d, numel = %d, length = %d\n', ...
        size(C,1), size(C,2), numel(C), length(C))

fprintf('norm(C)  (Frobenius):  %.4f\n\n', norm(C))

% --- 7. Practical: DFT matrix ---
%
% The N-point DFT matrix W has elements W(m,n) = exp(-2*pi*i*(m-1)*(n-1)/N).
% Multiplying a column vector by W gives its DFT.
%
% Built using vectorized outer product — no element-by-element loop needed.
% Property: W * W' = N * I  (columns are mutually orthogonal).
%
% Values are rounded to 4 decimal places; floating-point noise near zero
% (|x| < 1e-12) would otherwise appear as tiny non-zero entries.

fprintf('=== 7. DFT matrix (N=4) ===\n')

N  = 4;
mn = (0:N-1)' * (0:N-1);          % N×N real outer-product matrix of indices
W  = cos(-2*pi/N * mn) + i * sin(-2*pi/N * mn);

fprintf('real(W):\n')
disp(round(real(W) * 1e4) / 1e4)

fprintf('imag(W):\n')
disp(round(imag(W) * 1e4) / 1e4)

% Verify orthogonality: real(W * W') should equal N * eye(N)
WWh = W * W';
fprintf('real(W * W'')  (should be %d * I):\n', N)
disp(round(real(WWh) * 1e4) / 1e4)

% Apply DFT matrix to [1 2 3 4]' and compare expected result
x   = [1; 2; 3; 4];
X   = W * x;
fprintf('DFT of [1 2 3 4] via matrix multiply:\n')
fprintf('  real part:  ')
disp(round(real(X)' * 1e4) / 1e4)
fprintf('  imag part:  ')
disp(round(imag(X)' * 1e4) / 1e4)
% Expected: real = [10, -2, -2, -2], imag = [0, 2, 0, -2]
