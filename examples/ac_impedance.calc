% AC circuit — impedance and phase angle
%
% For a series RL circuit at a given frequency:
%   Z = sqrt(R^2 + X^2)   =>  hypot(R, X)
%   phi = atan2(X, R)     phase angle in radians
%
% Usage: ccalc ac_impedance.ccalc

R = 470;          % resistance, Ohm
f = 1000;         % frequency, Hz
L = 0.1;          % inductance, H
X = 2 * pi * f * L;   % inductive reactance

fprintf('Resistance R (Ohm): %d\n', R)
fprintf('Reactance X (Ohm): %.2f\n', X)

% --- Impedance magnitude: hypot avoids overflow vs sqrt(R^2 + X^2) ---
Z = hypot(R, X);
fprintf('Impedance |Z| (Ohm): %.2f\n', Z)

% --- Phase angle ---
phi_rad = atan2(X, R);
fprintf('Phase angle (rad): %.2f\n', phi_rad)

phi_deg = phi_rad * 180 / pi;
fprintf('Phase angle (deg): %.2f\n', phi_deg)

% Normalize to [0, 360) with mod
phi_norm = mod(phi_deg, 360);
fprintf('Normalized angle (deg): %.2f\n', phi_norm)

% --- Power factor ---
pf = cos(phi_rad);
pf = max(0, min(pf, 1));   % clamp to [0, 1]
fprintf('Power factor: %.2f\n', pf)

% --- Level in dB: 20 * log10(Z) ---
z_db = 20 * log(Z);
fprintf('|Z| in dB: %.2f\n', z_db)

% --- Bit width needed to represent Z (rounded up): log(Z, 2) ---
bits = ceil(log(Z, 2));
fprintf('Bit width for |Z|: %d\n', bits)

% --- Phase remainder after 90-degree steps (rem keeps sign) ---
phase_rem = rem(phi_deg, 90);
fprintf('Phase remainder mod 90 deg (rem): %.2f\n', phase_rem)
