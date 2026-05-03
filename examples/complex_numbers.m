% complex_numbers.calc — Phase 8 feature demo
%
% Covers: imaginary unit i/j, complex arithmetic (+,-,*,/,^),
%         built-ins (real, imag, abs, angle, conj, complex, isreal),
%         comparison, and a practical AC-circuit example.
%
% Usage: ccalc complex_numbers.calc

% --- 1. The imaginary unit ---

fprintf('=== 1. Imaginary unit ===\n')

fprintf('i = ')
disp(i)
fprintf('j = ')
disp(j)
fprintf('i^2 = ')
disp(i^2)
fprintf('i^3 = ')
disp(i^3)
fprintf('i^4 = ')
disp(i^4)

% --- 2. Creating complex numbers ---

fprintf('\n=== 2. Creating complex numbers ===\n')

z1 = 3 + 4*i;
fprintf('z1 = 3 + 4i  =>  ')
disp(z1)

z2 = complex(1, -2);
fprintf('z2 = complex(1,-2)  =>  ')
disp(z2)

% Pure imaginary
z3 = 5*i;
fprintf('z3 = 5i  =>  ')
disp(z3)

% Complex with negative imaginary part
z4 = 2 - 3*i;
fprintf('z4 = 2 - 3i  =>  ')
disp(z4)

% --- 3. Arithmetic ---

fprintf('\n=== 3. Arithmetic ===\n')

fprintf('z1 + z2  =>  ')
disp(z1 + z2)

fprintf('z1 - z2  =>  ')
disp(z1 - z2)

fprintf('z1 * z2  =>  ')
disp(z1 * z2)

fprintf('z1 / z2  =>  ')
disp(z1 / z2)

% Multiplying conjugate pair always gives a real result
fprintf('z1 * conj(z1) = |z1|^2  =>  ')
disp(z1 * conj(z1))

% Mixed scalar-complex
fprintf('2 * z1  =>  ')
disp(2 * z1)
fprintf('z1 + 10  =>  ')
disp(z1 + 10)

% --- 4. Polar form: abs and angle ---

fprintf('\n=== 4. Polar form ===\n')

fprintf('z1 = 3 + 4i\n')
fprintf('  abs(z1)   = |z1|     =>  %.2f\n', abs(z1))
fprintf('  angle(z1) = arg(z1)  =>  %.4f\n', angle(z1))
fprintf('  angle in degrees     =>  %.2f\n', angle(z1) * 180 / pi)

% Reconstruct from polar form: r * (cos(theta) + i*sin(theta))
r = abs(z1);
theta = angle(z1);
fprintf('Reconstructed: r*(cos(t)+i*sin(t))  =>  ')
disp(complex(r * cos(theta), r * sin(theta)))

% --- 5. Built-in functions ---

fprintf('\n=== 5. Built-in functions ===\n')

z = 3 + 4*i;
fprintf('z = 3 + 4i\n')

fprintf('  real(z)   =>  %.2f\n', real(z))
fprintf('  imag(z)   =>  %.2f\n', imag(z))
fprintf('  conj(z)   =>  ')
disp(conj(z))
fprintf('  abs(z)    =>  %.2f\n', abs(z))
fprintf('  angle(z)  =>  %.4f\n', angle(z))
fprintf('  isreal(z) =>  %d\n', isreal(z))

% Scalar is always real
fprintf('  isreal(5) =>  %d\n', isreal(5))

% imag of a real scalar
fprintf('  imag(7)   =>  %.2f\n', imag(7))

% --- 6. Conjugate transpose ---

fprintf('\n=== 6. Conjugate transpose ===\n')

fprintf('z = 3 + 4i\n')
fprintf("z' (conjugate)  =>  ")
disp(z')

% Conjugate via conj
fprintf('conj(z)         =>  ')
disp(conj(z))

% --- 7. Powers ---

fprintf('\n=== 7. Powers ===\n')

% Integer powers use exact repeated multiplication
fprintf('(1 + i)^0  =>  ')
disp((1 + i)^0)
fprintf('(1 + i)^1  =>  ')
disp((1 + i)^1)
fprintf('(1 + i)^2  =>  ')
disp((1 + i)^2)
fprintf('(1 + i)^4  =>  ')
disp((1 + i)^4)
fprintf('(1 + i)^-1 =>  ')
disp((1 + i)^-1)

% General power via polar form
fprintf('i^0.5 = sqrt(i)  =>  ')
disp(i^0.5)

% Euler's formula: e^(i*pi) + 1 ≈ 0
% (tiny imaginary residual from floating-point sin(pi) ≈ 1.22e-16)
fprintf('\nEuler: e^(i*pi) + 1  =>  ')
disp(e^(i * pi) + 1)

% --- 8. Comparison ---

fprintf('\n=== 8. Comparison ===\n')

a = 3 + 4*i;
b = 3 + 4*i;
c = 3 - 4*i;

fprintf('a = 3+4i,  b = 3+4i,  c = 3-4i\n')
fprintf('a == b  =>  %d\n', a == b)
fprintf('a == c  =>  %d\n', a == c)
fprintf('a ~= c  =>  %d\n', a ~= c)

% --- 9. Practical: AC circuit with complex impedance ---

fprintf('\n=== 9. AC circuit (series RLC) ===\n')
fprintf('Using complex impedance: Z = R + j*(X_L - X_C)\n\n')

R   = 100;          % resistance, Ohm
L   = 0.05;         % inductance, H
C   = 1e-6;         % capacitance, F
f   = 1000;         % frequency, Hz
w   = 2 * pi * f;   % angular frequency, rad/s

X_L = w * L;        % inductive reactance
X_C = 1 / (w * C);  % capacitive reactance

fprintf('R   = %d Ohm\n', R)
fprintf('X_L = %.2f Ohm\n', X_L)
fprintf('X_C = %.2f Ohm\n', X_C)

% Complex impedance
Z = complex(R, X_L - X_C);
fprintf('Z   = R + j*(X_L - X_C)  =>  ')
disp(Z)

% Magnitude and phase
fprintf('|Z|  (Ohm)      =>  %.2f\n', abs(Z))
fprintf('phi  (deg)      =>  %.2f\n', angle(Z) * 180 / pi)

% Admittance Y = 1/Z
Y = inv(Z);
fprintf('Y = 1/Z         =>  ')
disp(Y)

% Resonant frequency: X_L = X_C  =>  w0 = 1/sqrt(L*C)
w0 = 1 / (L * C)^0.5;
f0 = w0 / (2 * pi);
fprintf('\nResonant frequency f0 (Hz) =>  %.2f\n', f0)

% At resonance, imaginary part of Z vanishes
Z0 = complex(R, 0);
fprintf('Z at resonance              =>  ')
disp(Z0)
