% extended_control_flow.calc — Phase 11.5 (switch, do...until, run/source)
%
% Topic: practical examples of the extended control-flow forms added in v0.16.
%        All expected values are annotated for easy verification.
%
% Usage: run from the examples/ directory
%   ccalc extended_control_flow.calc

% ── 1. switch — integer dispatch ─────────────────────────────────────────────
%
% Map a process exit code to a human-readable status string.

fprintf('=== 1. Exit-code classifier ===\n')

for code = [0 1 2 127]
  switch code
    case 0
      status = 'success';
    case 1
      status = 'generic error';
    case 2
      status = 'misuse of shell command';
    otherwise
      status = 'unknown status';
  end
  fprintf('  exit %3d  ->  %s\n', code, status)
end

% Expected output:
%   exit   0  ->  success
%   exit   1  ->  generic error
%   exit   2  ->  misuse of shell command
%   exit 127  ->  unknown status

% ── 2. switch — string dispatch ──────────────────────────────────────────────
%
% Convert an SI unit abbreviation to the conversion factor relative to metres.

fprintf('\n=== 2. Unit abbreviation → conversion factor ===\n')

% Iterate over a proxy index; each case assigns both the label and the factor.
for idx = 1:5
  switch idx
    case 1
      unit = 'm';    factor = 1.0;
    case 2
      unit = 'km';   factor = 1000.0;
    case 3
      unit = 'cm';   factor = 0.01;
    case 4
      unit = 'mi';   factor = 1609.344;
    case 5
      unit = '???';  factor = -1.0;
  end
  if factor > 0
    fprintf('  1 %s  =  %g m\n', unit, factor)
  else
    fprintf('  %s: unknown unit\n', unit)
  end
end

% Expected output:
%   1 m   =  1 m
%   1 km  =  1000 m
%   1 cm  =  0.01 m
%   1 mi  =  1609.34 m
%   ???: unknown unit

% ── 3. switch — no match + otherwise ─────────────────────────────────────────
%
% Season lookup from month number.

fprintf('\n=== 3. Month → season ===\n')

for month = [1 4 7 10 13]
  switch month
    case 1
      season = 'Winter';
    case 2
      season = 'Winter';
    case 3
      season = 'Spring';
    case 4
      season = 'Spring';
    case 5
      season = 'Spring';
    case 6
      season = 'Summer';
    case 7
      season = 'Summer';
    case 8
      season = 'Summer';
    case 9
      season = 'Autumn';
    case 10
      season = 'Autumn';
    case 11
      season = 'Autumn';
    case 12
      season = 'Winter';
    otherwise
      season = 'invalid month';
  end
  fprintf('  month %2d  ->  %s\n', month, season)
end

% Expected output:
%   month  1  ->  Winter
%   month  4  ->  Spring
%   month  7  ->  Summer
%   month 10  ->  Autumn
%   month 13  ->  invalid month

% ── 4. do...until — smallest power of 2 that is >= n ─────────────────────────
%
% Because the body always runs at least once, the seed value p = 1 is safe
% even when n <= 1.

fprintf('\n=== 4. Smallest power of 2 >= n ===\n')

for n = [1 7 100 1000]
  p = 1;
  do
    p *= 2;
  until (p >= n)
  fprintf('  n = %4d  ->  2^k = %4d\n', n, p)
end

% Expected output:
%   n =    1  ->  2^k =    2
%   n =    7  ->  2^k =    8
%   n =  100  ->  2^k =  128
%   n = 1000  ->  2^k = 1024

% ── 5. do...until — digit sum ────────────────────────────────────────────────
%
% Sum the decimal digits of a number.  The loop runs at least once, which
% correctly handles the edge case num = 0 (digit sum = 0).

fprintf('\n=== 5. Digit sums ===\n')

for num = [9876 12345 1001 7]
  digit_sum = 0;
  n = num;
  do
    digit_sum += mod(n, 10);
    n = floor(n / 10);
  until (n == 0)
  fprintf('  digits(%5d) = %d\n', num, digit_sum)
end

% Expected output:
%   digits( 9876) = 30
%   digits(12345) = 15
%   digits( 1001) = 2
%   digits(    7) = 7

% ── 6. do...until with break — find first prime in a range ───────────────────

fprintf('\n=== 6. First prime after 50 ===\n')

candidate = 50;
found_prime = 0;
do
  candidate++;
  is_prime = 1;
  d = 2;
  while d * d <= candidate
    if mod(candidate, d) == 0
      is_prime = 0;
      break
    end
    d++;
  end
  if is_prime
    found_prime = candidate;
    break
  end
until (candidate > 1000)   % safety limit

fprintf('  first prime after 50: %d   (exact: 53)\n', found_prime)

% ── 7. run() — source a helper script ────────────────────────────────────────
%
% euclid_helper.calc reads 'a' and 'b' from the workspace, computes their
% greatest common divisor, and writes the result back as 'g'.
% Variables defined in the helper persist in this script's workspace
% (MATLAB `run` semantics — same scope, not a function call).

fprintf('\n=== 7. run() — Euclidean GCD via helper script ===\n')

a = 252; b = 105;
run('euclid_helper')
fprintf('  gcd(252, 105) = %d   (exact: 21)\n', g)

a = 360; b = 450;
run('euclid_helper')
fprintf('  gcd(360, 450) = %d   (exact: 90)\n', g)

a = 17; b = 13;
run('euclid_helper')
fprintf('  gcd(17, 13)   = %d    (exact: 1 — coprime)\n', g)

% ── 8. source() — Octave alias for run() ─────────────────────────────────────

fprintf('\n=== 8. source() — alias for run() ===\n')

a = 48; b = 180;
source('euclid_helper')
fprintf('  gcd(48, 180) = %d   (exact: 12)\n', g)

a = 100; b = 75;
source('euclid_helper')
fprintf('  gcd(100, 75) = %d   (exact: 25)\n', g)
