% Phase 24 — Polynomial operations and interpolation

% ── Polynomial representation ──────────────────────────────────────────────
% Row vector of coefficients, highest degree first.
% p(x) = x^2 - 3x + 2  →  [1  -3  2]

% ── Evaluation (polyval) ───────────────────────────────────────────────────
p = [1 0 1];           % x^2 + 1
disp('polyval([1 0 1], [0 1 2 3]):')
disp(polyval(p, [0 1 2 3]))   % [1 2 5 10]

% ── Least-squares fitting (polyfit) ────────────────────────────────────────
x = [0 1 2 3 4];
y = [1 2 5 10 17];
p_fit = polyfit(x, y, 2);
fprintf('polyfit degree-2 coefficients: [%.6f %.6f %.6f]\n', ...
        p_fit(1), p_fit(2), p_fit(3))
% Should be approximately [1.0, 0.0, 1.0]  (x^2 + 1)

% ── Roots (roots) ──────────────────────────────────────────────────────────
p_quad = [1 1 6];    % x^2 + x + 6  (complex roots: (-1 ± sqrt(1-24))/2)
disp('roots([1 1 6]):')
disp(roots(p_quad))

% All-real case
p_real = [1 4 4];    % (x+2)^2  →  double root at -2
disp('roots([1 4 4]):')
disp(roots(p_real))

% ── Monic polynomial from roots (poly) ────────────────────────────────────
r = [1 2 3];
disp('poly([1 2 3]):')
disp(poly(r))    % [1 -6 11 -6]

% Characteristic polynomial of a matrix
A = [2 1; 0 3];
disp('poly([2 1; 0 3]):')
disp(poly(A))    % [1 -5 6]  ( (λ-2)(λ-3) )

% ── Convolution (conv) ─────────────────────────────────────────────────────
a = [1 2 3];
b = [1 1];
disp('conv([1 2 3], [1 1]):')
disp(conv(a, b))   % [1 3 5 3]

% ── Deconvolution (deconv) ─────────────────────────────────────────────────
c = [1 3 5 3];
[q, r] = deconv(c, b);
fprintf('deconv([1 3 5 3], [1 1]):\n')
fprintf('  q = [%g %g %g]\n', q(1), q(2), q(3))
fprintf('  r = [%g %g %g %g]\n', r(1), r(2), r(3), r(4))

% ── Interpolation (interp1) ────────────────────────────────────────────────
xi = [0 1 2 3];
yi = [0 1 4 9];

% Linear (default)
fprintf('interp1 linear at 1.5: %.4f\n', interp1(xi, yi, 1.5))

% Nearest neighbour
fprintf('interp1 nearest at 1.5: %.4f\n', interp1(xi, yi, 1.5, 'nearest'))

% Previous (zero-order hold)
fprintf('interp1 previous at 1.5: %.4f\n', interp1(xi, yi, 1.5, 'previous'))

% Extrapolation returns NaN
v = interp1(xi, yi, 99);
fprintf('interp1 at 99 (out of range): %s\n', num2str(v))

% Curve fitting example
x_data = [0 0.5 1 1.5 2 2.5 3];
y_data = [0 0.2 1 2.3 4.1 6.2 9.0];
p3 = polyfit(x_data, y_data, 2);
fprintf('Fit to noisy x^2 data: a=%.4f b=%.4f c=%.4f\n', ...
        p3(1), p3(2), p3(3))
