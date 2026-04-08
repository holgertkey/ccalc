% Resistor network calculations (Ohm's law)
% Series:   R_total = R1 + R2 + R3
% Parallel: R_total = 1 / (1/R1 + 1/R2)
%
% Usage: ccalc resistors.ccalc

% --- Series combination: R1=100, R2=220, R3=470 ---
r_series = 100 + 220 + 470;
fprintf('Series resistance (Ohm): %d\n', r_series)

% Voltage divider: V_out across R3 with V_in = 12V
fprintf('Voltage divider V_out (V): %.2f\n', 12 * 470 / r_series)

% --- Parallel combination of R1=100 and R2=220 ---
r_parallel = (1/100 + 1/220) ^ -1;
fprintf('Parallel resistance (Ohm): %.2f\n', r_parallel)

% Power dissipated at 5V: P = V^2 / R
fprintf('Power dissipated (W): %.2f\n', 5^2 / r_parallel)
