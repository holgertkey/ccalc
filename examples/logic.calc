% logic.calc — Phase 7 (comparison and logical operators) demo
%
% Topic: ADC input validation and soft clipping
%
% An analog-to-digital converter (ADC) accepts voltages in [0, 3.3] V.
% We have a batch of raw sensor readings. We need to:
%   1. Identify which readings are within the valid range.
%   2. Flag readings that are out of range (too low or too high).
%   3. Produce a soft-clipped output: in-range values pass through;
%      out-of-range values are zeroed (masked out).
%
% Usage: ccalc logic.calc

% --- 1. Scalar comparison basics ---

fprintf('=== 1. Scalar comparisons ===\n')

v_in  = 2.7;    % input voltage
v_ref = 3.3;    % ADC reference (maximum valid input)

fprintf('v_in  = %.2f\n', v_in)
fprintf('v_ref = %.2f\n', v_ref)

% Each comparison returns 1.0 (true) or 0.0 (false)
fprintf('v_in < v_ref  (in range?)   : %d\n', v_in < v_ref)
fprintf('v_in == v_ref (at maximum?) : %d\n', v_in == v_ref)
fprintf('v_in > v_ref  (over-range?) : %d\n', v_in > v_ref)

% --- 2. Logical NOT ---

fprintf('\n=== 2. Logical NOT ===\n')

in_range = v_in >= 0 && v_in <= v_ref;
fprintf('in_range  (0 <= v_in <= 3.3): %d\n', in_range)
fprintf('fault     = ~in_range       : %d\n', ~in_range)

% Over-range example
v_bad     = 4.1;
in_range_bad = v_bad >= 0 && v_bad <= v_ref;
fprintf('\nv_bad = %.2f\n', v_bad)
fprintf('in_range  (v_bad)           : %d\n', in_range_bad)
fprintf('fault     = ~in_range_bad   : %d\n', ~in_range_bad)

% --- 3. Combining conditions with && and || ---

fprintf('\n=== 3. Combined conditions (&&, ||) ===\n')

v_lo = 0.0;   % lower bound
v_hi = 3.3;   % upper bound

% Check three readings individually
r1 = 1.5;
r2 = -0.2;
r3 = 3.1;

fprintf('r1 = 1.5  → valid: %d\n', r1 >= v_lo && r1 <= v_hi)
fprintf('r2 = -0.2 → valid: %d\n', r2 >= v_lo && r2 <= v_hi)
fprintf('r3 = 3.1  → valid: %d\n', r3 >= v_lo && r3 <= v_hi)

% Any of the three invalid?
any_invalid = ~(r1 >= v_lo && r1 <= v_hi) || ~(r2 >= v_lo && r2 <= v_hi) || ~(r3 >= v_lo && r3 <= v_hi);
fprintf('\nAny reading out of range? : %d\n', any_invalid)

% All three valid?
all_valid = (r1 >= v_lo && r1 <= v_hi) && (r2 >= v_lo && r2 <= v_hi) && (r3 >= v_lo && r3 <= v_hi);
fprintf('All readings valid?        : %d\n', all_valid)

% --- 4. Element-wise comparison on a vector ---

fprintf('\n=== 4. Element-wise comparison ===\n')

% Batch of 8 ADC readings (some out of range)
readings = [-0.1, 0.5, 1.2, 3.3, 3.8, 2.0, -0.5, 1.8];
fprintf('readings:\n')
disp(readings)

% Create boolean masks (1 = condition met, 0 = not)
mask_lo = readings >= 0;       % not under-range
mask_hi = readings <= 3.3;     % not over-range
fprintf('readings >= 0   (not under-range):\n')
disp(mask_lo)

fprintf('readings <= 3.3 (not over-range):\n')
disp(mask_hi)

% --- 5. Combining masks and soft clipping ---

fprintf('\n=== 5. Soft clipping via mask multiplication ===\n')

% A reading is valid if BOTH bounds are satisfied.
% Since the masks are 0/1 vectors, element-wise product = logical AND.
valid   = mask_lo .* mask_hi;
invalid = ~valid;              % element-wise NOT
fprintf('valid   mask:\n')
disp(valid)
fprintf('invalid mask:\n')
disp(invalid)

% Soft-clipped output: keep valid readings, zero out the rest
clipped = readings .* valid;
fprintf('Soft-clipped output (invalid → 0):\n')
disp(clipped)

% --- 6. Resistor tolerance check ---

fprintf('\n=== 6. Resistor tolerance check ===\n')

% 1 kOhm ±5% resistors measured from a reel
r_nominal = 1000;
tol       = 0.05;   % 5 %

r_measured = [985, 1002, 1048, 997, 960, 1019, 1055, 1001];
fprintf('r_measured:\n')
disp(r_measured)

r_lo = r_nominal * (1 - tol);   % 950 Ohm
r_hi = r_nominal * (1 + tol);   % 1050 Ohm
fprintf('r_lo (950 Ohm): %.2f\n', r_lo)
fprintf('r_hi (1050 Ohm): %.2f\n', r_hi)

in_tol = (r_measured >= r_lo) .* (r_measured <= r_hi);
fprintf('In tolerance (1 = pass, 0 = fail):\n')
disp(in_tol)

% Failing resistors zeroed out, passing values kept
passed = r_measured .* in_tol;
fprintf('Passing values (failing → 0):\n')
disp(passed)

% Last check: is the first resistor within spec AND not equal to nominal?
r1_ok     = r_measured(1) >= r_lo && r_measured(1) <= r_hi;
r1_not_nom = r_measured(1) ~= r_nominal;
fprintf('\nr_measured(1) = %d\n', r_measured(1))
fprintf('Within spec?         : %d\n', r1_ok)
fprintf('Not exactly nominal? : %d\n', r1_not_nom)
fprintf('Within spec AND not nominal: %d\n', r1_ok && r1_not_nom)
