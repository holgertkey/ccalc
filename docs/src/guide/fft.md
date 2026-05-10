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
Returns a cell array where each element is a complex number `re+im·i`.

```matlab
x = [1 2 3 4];
X = fft(x)
% X{1} = 10+0i   (DC component: sum of all samples)
% X{2} = -2+2i
% X{3} = -2+0i
% X{4} = -2-2i
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
When all imaginary parts are negligibly small (< 1e-12), returns a real matrix
instead of a cell array:

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

```matlab
fs = 1000;                      % sampling rate  (Hz)
t  = 0:1/fs:1-1/fs;            % 1 s of samples
x  = sin(2*pi*50*t) + 0.5*sin(2*pi*120*t);  % 50 Hz + 120 Hz signal

X  = fft(x);
n  = length(t);
f  = fftfreq(n, 1/fs);

% Compute one-sided power spectrum amplitude:
% (element-wise abs of each cell element requires a loop for now)
% See: abs of complex result after Phase 27 introduces ComplexMatrix.
```

---

## Summary

| Function | Args | Feature flag |
|----------|------|--------------|
| `fft(x)` | real vector | `fft` |
| `fft(x, n)` | real vector, length | `fft` |
| `ifft(X)` | cell of complex | `fft` |
| `fftshift(x)` | real matrix | always |
| `ifftshift(x)` | real matrix | always |
| `fftfreq(n, d)` | count, spacing | always |
