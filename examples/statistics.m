% statistics.calc — Phase 17 feature demo
%
% Covers: rand, randn, randi, rng,
%         std, var, cov, median, mode,
%         hist, histc,
%         prctile, iqr, zscore,
%         erf, erfc, normcdf, normpdf
%
% Usage: ccalc statistics.calc

% Fix the seed so every run produces the same output.
rng(2025)

% ── 1. Random number generation ───────────────────────────────────────────────

fprintf('=== 1. Random number generation ===\n')

% Uniform [0, 1)
u = rand(1, 6);
fprintf('rand(1,6)  = ')
disp(u)

% Standard normal
z = randn(1, 6);
fprintf('randn(1,6) = ')
disp(z)

% Random integers in [1, 20]
d = randi(20, 1, 8);
fprintf('randi(20,1,8) = ')
disp(d)

% rng(seed) makes runs reproducible
rng(42)
a = rand(1, 3);
rng(42)
b = rand(1, 3);
fprintf('Same seed → same values: %d\n', all(a == b))

% ── 2. Simulated dataset ──────────────────────────────────────────────────────

fprintf('\n=== 2. Simulated dataset (n=200 normal samples) ===\n')

% Build a 200-element column vector from randn.
% (Loop used because randn(200,1) is a single call, but this shows both idioms.)
rng(7)
n = 200;
mu_true = 50;
sigma_true = 10;

data = zeros(1, n);
for k = 1:n
  data(k) = mu_true + sigma_true * randn();
end

fprintf('n          = %d\n', n)
fprintf('true mu    = %.1f,  estimated mean = %.4f\n', mu_true, mean(data))
fprintf('true sigma = %.1f,  estimated std  = %.4f\n', sigma_true, std(data))
fprintf('variance   = %.4f  (= std^2: %.4f)\n', var(data), std(data)^2)
fprintf('median     = %.4f\n', median(data))
fprintf('min / max  = %.4f / %.4f\n', min(data), max(data))

% ── 3. Percentiles and spread ─────────────────────────────────────────────────

fprintf('\n=== 3. Percentiles and spread ===\n')

p5  = prctile(data, 5);
p25 = prctile(data, 25);
p50 = prctile(data, 50);
p75 = prctile(data, 75);
p95 = prctile(data, 95);

fprintf('P5  = %.2f\n', p5)
fprintf('P25 = %.2f\n', p25)
fprintf('P50 = %.2f  (median)\n', p50)
fprintf('P75 = %.2f\n', p75)
fprintf('P95 = %.2f\n', p95)

% Inter-quartile range
q = iqr(data);
fprintf('IQR = %.2f  (P75 - P25 = %.2f)\n', q, p75 - p25)

% Outlier fence (1.5 × IQR rule)
lo_fence = p25 - 1.5 * q;
hi_fence = p75 + 1.5 * q;
n_outliers = sum(data < lo_fence | data > hi_fence);
fprintf('Outlier fences: [%.2f, %.2f]\n', lo_fence, hi_fence)
fprintf('Outliers found: %d\n', n_outliers)

% ── 4. Standardisation (z-scores) ─────────────────────────────────────────────

fprintf('\n=== 4. Z-scores ===\n')

z_data = zscore(data);
fprintf('After zscore: mean = %.6f  (should be ~0)\n', mean(z_data))
fprintf('             std  = %.6f  (should be ~1)\n', std(z_data))

% z-score of a specific value (how many sigmas from the mean?)
x_test = 75;
z_test = (x_test - mean(data)) / std(data);
fprintf('x = %d is %.2f standard deviations above the mean\n', x_test, z_test)

% ── 5. ASCII histogram ─────────────────────────────────────────────────────────

fprintf('\n=== 5. Histogram of simulated data ===\n')
hist(data, 12)

% histc: count how many values fall in user-defined bins
edges = [20 30 40 50 60 70 80];
counts = histc(data, edges);
fprintf('Bin counts (edges 20:10:80):\n')
for k = 1:length(edges)
  fprintf('  [%d, %d): %d\n', edges(k), edges(k) + 10, counts(k))
end

% ── 6. Mode on discrete data ──────────────────────────────────────────────────

fprintf('\n=== 6. Mode on integer data ===\n')

rolls = randi(6, 1, 60);         % 60 dice rolls
fprintf('60 dice rolls — mode (most frequent face): %d\n', mode(rolls))

face_counts = zeros(1, 6);
for f = 1:6
  face_counts(f) = sum(rolls == f);
end
fprintf('Face frequencies: ')
disp(face_counts)

% ── 7. Covariance of two correlated variables ─────────────────────────────────

fprintf('\n=== 7. Covariance / correlation ===\n')

% Build two correlated variables: y = 3*x + noise
rng(99)
m = 50;
x = randn(m, 1);
y = 3 * x + 0.5 * randn(m, 1);

% Stack into m×2 matrix; cov() returns 2×2 covariance matrix
XY = [x, y];
C = cov(XY);
fprintf('Cov matrix (x, 3x+noise):\n')
disp(C)

% Pearson r from covariance matrix
r = C(1, 2) / sqrt(C(1, 1) * C(2, 2));
fprintf('Pearson r = %.4f  (expect ~0.98 for y = 3x + small noise)\n', r)

% ── 8. Normal distribution functions ──────────────────────────────────────────

fprintf('\n=== 8. Normal distribution functions ===\n')

% PDF peak and symmetry
fprintf('normpdf(0)        = %.6f  (peak of std normal)\n', normpdf(0))
fprintf('normpdf(1)        = %.6f\n', normpdf(1))
fprintf('normpdf(-1)       = %.6f  (symmetric)\n', normpdf(-1))

% CDF: probability that a std normal < x
fprintf('\nnormcdf(0)        = %.4f  (50%% probability)\n', normcdf(0))
fprintf('normcdf(1)        = %.4f  (68%% rule: P(-1<Z<1) = %.4f)\n', ...
  normcdf(1), normcdf(1) - normcdf(-1))
fprintf('normcdf(2)        = %.4f  (95%% rule: P(-2<Z<2) = %.4f)\n', ...
  normcdf(2), normcdf(2) - normcdf(-2))
fprintf('normcdf(3)        = %.4f  (99.7%% rule: P(-3<Z<3) = %.4f)\n', ...
  normcdf(3), normcdf(3) - normcdf(-3))

% General: probability that a N(50, 10) sample is between 40 and 60
p_40_60 = normcdf(60, 50, 10) - normcdf(40, 50, 10);
fprintf('\nP(40 < N(50,10) < 60) = %.4f  (expect ~0.6827)\n', p_40_60)

% How does our simulated dataset compare?
p_sim = mean(data >= 40 & data <= 60);
fprintf('Fraction of simulated data in [40, 60]: %.4f\n', p_sim)

% ── 9. Error function ─────────────────────────────────────────────────────────

fprintf('\n=== 9. Error function (erf / erfc) ===\n')

fprintf('erf(0)   = %.4f  (odd function: erf(0)=0)\n', erf(0))
fprintf('erf(1)   = %.4f\n', erf(1))
fprintf('erf(inf) = %.4f  (limit is 1)\n', erf(1e10))
fprintf('erfc(x)  = 1 - erf(x), so erf(1) + erfc(1) = %.10f\n', erf(1) + erfc(1))

% erf relates to normcdf: normcdf(x) = 0.5*(1 + erf(x/sqrt(2)))
x_val = 1.5;
from_erf    = 0.5 * (1 + erf(x_val / sqrt(2)));
from_normcdf = normcdf(x_val);
fprintf('\nnormcdf(1.5) via erf:     %.10f\n', from_erf)
fprintf('normcdf(1.5) direct:      %.10f\n', from_normcdf)
fprintf('difference:               %.2e\n', abs(from_erf - from_normcdf))
