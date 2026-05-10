% Phase 26 — FFT & signal processing
%
% Covers: fft, fft(x,n), ifft, fftshift, ifftshift, fftfreq
%
% fft and ifft require the fft feature flag at build time:
%   cargo build --release --features fft
%
% fftshift, ifftshift, and fftfreq are always available.

% --- 1. Forward FFT ---
%
% fft(x) returns a cell array of complex numbers.
% X{1} is the DC component, equal to the sum of all input samples.

x = [1 2 3 4];
X = fft(x);
fprintf('fft([1 2 3 4]):\n')
for k = 1:4
  fprintf('  X{%d} = ', k)
  disp(X{k})
end
% X{1} = 10       (DC component: sum of all samples)
% X{2} = -2 + 2i
% X{3} = -2
% X{4} = -2 - 2i

% --- 2. Inverse FFT roundtrip ---
%
% When all imaginary parts are negligibly small, ifft returns a real matrix.

y = ifft(X);
fprintf('\nifft(fft([1 2 3 4])) roundtrip:\n')
disp(y)   % [1 2 3 4]

% --- 3. Zero-padded FFT ---
%
% fft(x, n) pads x with zeros to length n before transforming.
% A longer FFT gives a finer frequency grid over the same signal.

X8 = fft(x, 8);
fprintf('fft([1 2 3 4], 8) — 8-point zero-padded:\n')
for k = 1:8
  fprintf('  X8{%d} = ', k)
  disp(X8{k})
end

% --- 4. fftshift / ifftshift ---
%
% fftshift reorders the output so that the DC bin moves to the centre,
% giving a spectrum symmetric around 0 Hz.  ifftshift undoes the shift.

fprintf('\nfftshift([1 2 3 4 5 6]):\n')
disp(fftshift([1 2 3 4 5 6]))   % [4 5 6 1 2 3]

fprintf('ifftshift reverses the shift:\n')
disp(ifftshift([4 5 6 1 2 3]))  % [1 2 3 4 5 6]

fprintf('Odd length — fftshift([1 2 3 4 5]):\n')
disp(fftshift([1 2 3 4 5]))     % [4 5 1 2 3]

% --- 5. Frequency axis ---
%
% fftfreq(n, d) returns the DFT sample frequencies for n points with
% sample spacing d = 1/fs seconds.  Matches NumPy and MATLAB convention.

f = fftfreq(8, 1/1000);
fprintf('\nfftfreq(8, 1/1000) — bins in Hz:\n')
disp(f)              % [0 125 250 375 -500 -375 -250 -125]

fprintf('Centred with fftshift:\n')
disp(fftshift(f))    % [-500 -375 -250 -125 0 125 250 375]

% --- 6. Two-tone power spectrum ---
%
% Synthesize a signal with two pure tones:
%   10 Hz at amplitude 1.0, 25 Hz at amplitude 0.5.
% Sample at fs = 100 Hz for n = 100 points (1 second of data).
%
% Both tones fall exactly on FFT bins (frequency resolution = fs/n = 1 Hz),
% so there is no spectral leakage.
%
% For a real sine of amplitude A, the one-sided FFT magnitude is A * n / 2.

n  = 100;
fs = 100;
t  = (0:n-1) / fs;                            % time axis: 0, 0.01, ..., 0.99 s
s  = sin(2*pi*10*t) + 0.5*sin(2*pi*25*t);     % two-tone signal
S  = fft(s);

% Compute magnitude for each bin
mag = zeros(1, n);
for k = 1:n
  mag(k) = sqrt(real(S{k})^2 + imag(S{k})^2);
end

% Bin mapping (1-based, frequency resolution = 1 Hz):
%   10 Hz → bin 11,   25 Hz → bin 26
fprintf('\nTwo-tone signal: 10 Hz (A=1.0) + 25 Hz (A=0.5)  [fs=%d, n=%d]\n', fs, n)
fprintf('  Bin 11 @ 10 Hz :  |S| = %6.2f   (expected %5.2f)\n', mag(11), 1.0 * n / 2)
fprintf('  Bin 26 @ 25 Hz :  |S| = %6.2f   (expected %5.2f)\n', mag(26), 0.5 * n / 2)
