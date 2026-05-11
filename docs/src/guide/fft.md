# FFT & Signal Processing

ccalc provides FFT-based frequency analysis through five built-in functions.

`fft` and `ifft` require the `fft` feature flag:

> **Requires the `fft` feature:**
> ```bash
> cargo build --release --features fft
> ```
> Without this flag, calling `fft` or `ifft` returns an error message explaining
> how to enable the feature. `fftshift`, `ifftshift`, and `fftfreq` are always
> available.

---

## Forward FFT — `fft`

`fft(x)` computes the Discrete Fourier Transform (DFT) of real vector `x`.
Returns a **ComplexMatrix** (1×N row vector) where each element is a
complex number `re+im·i`. Access individual bins with `X(k)`.

```matlab
x = [1 2 3 4];
X = fft(x)
% X(1) = 10 + 0i   (DC component: sum of all samples)
% X(2) = -2 + 2i
% X(3) = -2 + 0i
% X(4) = -2 - 2i
```

### Zero-padded FFT

`fft(x, n)` pads `x` with zeros to length `n` before the transform (or truncates
if `n < length(x)`). Use to control the frequency resolution:

```matlab
X = fft([1 2 3 4], 8)   % 8-point FFT of a 4-sample signal
```

---

## Inverse FFT — `ifft`

`ifft(X)` computes the inverse DFT, normalised by 1/N.
Accepts a `ComplexMatrix` (as returned by `fft`).
When all imaginary parts are negligibly small (< 1e-12), returns a real matrix:

```matlab
x = [1 2 3 4];
X = fft(x);
y = ifft(X)   % → [1 2 3 4]  (real matrix; imaginary parts dropped)
```

---

## Shift DC to centre — `fftshift` / `ifftshift`

`fftshift(x)` performs a circular shift by `floor(N/2)` so that the DC
component (index 1) moves to the centre of the array. Used to produce a
zero-centred spectrum plot.

`ifftshift(x)` undoes the shift (`ceil(N/2)`).

```matlab
fftshift([1 2 3 4 5 6])      % → [4 5 6 1 2 3]
ifftshift([4 5 6 1 2 3])     % → [1 2 3 4 5 6]

fftshift([1 2 3 4 5])        % → [4 5 1 2 3]   (odd length)
ifftshift(fftshift([1 2 3 4 5]))  % → [1 2 3 4 5]
```

For 2-D matrices both dimensions are shifted simultaneously.

---

## Frequency axis — `fftfreq`

`fftfreq(n, d)` returns a 1×n row vector of DFT sample frequencies for `n`
points with sample spacing `d` seconds. The result is in cycles per unit of `d`.

```matlab
n  = 8;
fs = 1000;              % sampling rate in Hz
d  = 1/fs;             % sample spacing in seconds
f  = fftfreq(n, d)
% → [0 125 250 375 -500 -375 -250 -125]  Hz
```

The formula matches NumPy/MATLAB:

```
f = [0, 1, ..., floor((n-1)/2), -floor(n/2), ..., -1] / (n·d)
```

---

## Worked example — power spectrum

Two-tone signal: 10 Hz (amplitude 1.0) and 25 Hz (amplitude 0.5), sampled at
100 Hz for 100 points (1 second). Both tones land exactly on FFT bins
(frequency resolution = 1 Hz), so there is no spectral leakage.

For a real sine of amplitude A, the one-sided magnitude is `A × n/2`.

`fft` returns a `ComplexMatrix`, so `abs(S)` gives a real matrix of
element-wise magnitudes directly — no loop needed:

```matlab
n  = 100;
fs = 100;
t  = (0:n-1) / fs;                        % 0, 0.01, …, 0.99 s
s  = sin(2*pi*10*t) + 0.5*sin(2*pi*25*t);
S  = fft(s);
f  = fftfreq(n, 1/fs);

% abs() on a ComplexMatrix returns a real Matrix of element-wise magnitudes.
mag = abs(S);

% Bins: 10 Hz → bin 11, 25 Hz → bin 26  (1-based; resolution = 1 Hz)
fprintf('Bin 11 @ 10 Hz :  |S| = %.2f   (expected %.2f)\n', mag(11), 1.0 * n/2)
fprintf('Bin 26 @ 25 Hz :  |S| = %.2f   (expected %.2f)\n', mag(26), 0.5 * n/2)
% Bin 11 @ 10 Hz :  |S| = 50.00   (expected 50.00)
% Bin 26 @ 25 Hz :  |S| = 25.00   (expected 25.00)

% Centred spectrum view using fftshift
f_centred   = fftshift(f);
mag_centred = fftshift(mag);
```

---

## Summary

| Function | Args | Feature flag |
|----------|------|--------------|
| `fft(x)` | real vector | `fft` |
| `fft(x, n)` | real vector, length | `fft` |
| `ifft(X)` | ComplexMatrix | `fft` |
| `fftshift(x)` | real or complex matrix | always |
| `ifftshift(x)` | real or complex matrix | always |
| `fftfreq(n, d)` | count, spacing | always |
