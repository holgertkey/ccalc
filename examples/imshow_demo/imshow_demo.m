% imshow_demo.m — Phase 32f: image / imshow demonstration
%
% Shows three rendering modes:
%   1. image(Z)      — alias for imagesc (min/max scaled colormap)
%   2. imshow(Z)     — grayscale, values clamped to [0,1]
%   3. imshow(R,G,B) — per-channel RGB rendering

% ── Build a simple 8×8 gradient ──────────────────────────────────────────────
n = 8;
[X, Y] = meshgrid(linspace(0, 1, n), linspace(0, 1, n));
Z = X .* Y;

% ── image(Z) — alias for imagesc, active colormap applied ────────────────────
colormap('hot');
image(Z, 'imshow_demo_image.svg');

% ── imshow(Z) — grayscale with clamp (no scaling) ────────────────────────────
imshow(Z, 'imshow_demo_gray.svg');

% ── RGB channels: R=X, G=1-X, B=Y ───────────────────────────────────────────
R = X;
G = 1 - X;
B = Y;
imshow(R, G, B, 'imshow_demo_rgb.svg');

% ── ASCII mode (no file argument) ────────────────────────────────────────────
title('Grayscale gradient (ASCII)');
imshow(Z);
