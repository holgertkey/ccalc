# Complex Numbers

ccalc supports complex numbers using the same syntax as Octave/MATLAB.
No special mode is needed — `i` and `j` are always available as the
imaginary unit.

## Creating complex numbers

```
3 + 4i           % 3 + 4i   — Ni suffix (no space before i/j)
3 + 4*i          % same — explicit multiply also works
3 + 4*j          % j is also the imaginary unit
complex(3, 4)    % construct from real and imaginary parts
5i               % pure imaginary: 5i
2 - 3i           % 2 - 3i
```

**`Ni` suffix syntax:** any decimal number immediately followed by `i` or `j`
(no space, no further alphanumeric characters) is treated as a complex literal.
The tokenizer expands `4i` to `4 * i` — the imaginary unit `i` must be in
scope (it is always pre-seeded at startup).

## Arithmetic

All standard operators work on complex numbers:

```
z1 = 3 + 4*i
z2 = 1 - 2*i

z1 + z2          % 4 + 2i
z1 - z2          % 2 + 6i
z1 * z2          % 11 - 2i     (a+bi)(c+di) = (ac-bd) + (ad+bc)i
z1 / z2          % -1 + 2i
```

Mixing complex and real scalars works naturally:

```
z1 + 10          % 13 + 4i
2 * z1           % 6 + 8i
z1 ^ 2           % -7 + 24i
```

When the imaginary part of a result is exactly zero, the value is shown
and stored as a real scalar:

```
(1+i) * (1-i)    % 2   (not 2 + 0i)
```

## Powers

Integer powers use binary exponentiation for exact results:

```
i^2              % -1    (exact)
i^3              % -i    (exact)
i^4              %  1    (exact)
(1+i)^4          % -4
(1+i)^-1         % 0.5 - 0.5i
```

Non-integer powers use the polar form `exp((c+di)·ln(a+bi))`:

```
i^0.5            % 0.7071067812 + 0.7071067812i   (sqrt of i)
2^(1+i)          % 1.5384778027 + 1.2779225526i
```

## Polar form

Every complex number `z = re + im*i` has a polar representation
`z = r * (cos θ + i * sin θ)`, where `r = abs(z)` and `θ = angle(z)`:

```
z = 3 + 4*i
abs(z)           % 5          (modulus |z| = sqrt(3² + 4²))
angle(z)         % 0.9272...  (argument in radians)
angle(z) * 180/pi  % 53.13°  (in degrees)
```

Reconstruct from polar:

```
r = abs(z);
t = angle(z);
complex(r*cos(t), r*sin(t))   % 3 + 4i
```

## Built-in functions

| Function | Description |
|----------|-------------|
| `real(z)` | Real part (`real(5)` → 5, `real(3+4i)` → 3) |
| `imag(z)` | Imaginary part (`imag(5)` → 0, `imag(3+4i)` → 4) |
| `abs(z)` | Modulus (also works on real scalars and matrices) |
| `angle(z)` | Argument in radians |
| `conj(z)` | Complex conjugate: `re - im*i` |
| `complex(re, im)` | Construct from two real scalars |
| `isreal(z)` | `1` if imaginary part is zero, else `0` |

```
z = 3 + 4*i
real(z)          % 3
imag(z)          % 4
conj(z)          % 3 - 4i
abs(z)           % 5
angle(z)         % 0.927...
isreal(z)        % 0
isreal(5)        % 1
imag(7)          % 0
```

## Conjugate and plain transpose

The postfix `'` operator returns the **conjugate** of a complex scalar
(matching the matrix Hermitian-transpose convention):

```
z = 3 + 4i
z'               % 3 - 4i   conjugate — flips imaginary sign
conj(z)          % 3 - 4i   same result
```

The postfix `.'` operator returns the **plain** transpose — no conjugation:

```
z.'              % 3 + 4i   plain transpose — imaginary part unchanged
```

For real scalars and matrices `'` and `.'` give identical results.
The distinction only matters for complex values.

## Comparison

`==` and `~=` compare both real and imaginary parts:

```
(3 + 4*i) == (3 + 4*i)    % 1
(3 + 4*i) == (3 - 4*i)    % 0
(3 + 4*i) ~= (3 - 4*i)    % 1
```

Ordering operators (`<`, `>`, `<=`, `>=`) return an error for complex
numbers — ordering is not defined for the complex plane.

## Imaginary unit variables

`i` and `j` are pre-set to `0 + 1i` at startup. You can reassign them
(e.g. `i = 5` for a loop counter), in which case the original value is
no longer available until you restart ccalc.

## Limitations

Complex matrices (`[1+2i, 3+4i]`) are not yet supported and return an error.
Use scalar complex variables until matrix complex support is added (a future phase).

## Example

```
% Euler's identity: e^(i*pi) + 1 ≈ 0
e^(i * pi) + 1        % ≈ 0  (tiny floating-point residual from sin(π))

% Roots of x^2 + 1 = 0
x1 = i
x2 = -i

% AC impedance of a series RL circuit
R = 100; L = 0.05; f = 1000;
w = 2 * pi * f;
Z = complex(R, w * L)        % 100 + 314.159i
abs(Z)                        % impedance magnitude
angle(Z) * 180/pi             % phase angle in degrees
```

See `examples/complex_numbers.calc` for a complete annotated example.
