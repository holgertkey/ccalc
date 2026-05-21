% fill_area_polar_demo.m — Phase 30e: fill, area, polar, style strings
% Usage: ccalc fill_area_polar_demo.m

% ── fill: filled triangle polygon ────────────────────────────────────────────
tx = [0, 1, 0.5];
ty = [0, 0, 1];

% ASCII: bounding-box density fill approximation
fprintf('fill (ASCII):\n');
fill(tx, ty);

% File: plotters Polygon element with red fill
fill(tx, ty, 'r', 'examples/fill_area_polar_demo/out/fill_triangle.svg');
fprintf('Saved fill_triangle.svg\n');

% ── area: filled area under curve ────────────────────────────────────────────
x = linspace(0, 2*pi, 80);
y = sin(x) + 1;

% ASCII
fprintf('\narea (ASCII):\n');
area(x, y);

% File: area under a sine wave
area(x, y, 'b', 'examples/fill_area_polar_demo/out/area_sine.svg');
fprintf('Saved area_sine.svg\n');

% ── polar: circle r=1 ────────────────────────────────────────────────────────
theta = linspace(0, 2*pi, 200);
r = ones(size(theta));

% ASCII
fprintf('\npolar (circle, ASCII):\n');
polar(theta, r);

% File
polar(theta, r, 'examples/fill_area_polar_demo/out/polar_circle.svg');
fprintf('Saved polar_circle.svg\n');

% ── polar: rose curve r = |cos(2*theta)| ─────────────────────────────────────
theta2 = linspace(0, 2*pi, 360);
r2 = abs(cos(2 * theta2));

title('rose curve');
polar(theta2, r2, 'examples/fill_area_polar_demo/out/polar_rose.svg');
fprintf('Saved polar_rose.svg\n');

% ── plot with style string ────────────────────────────────────────────────────
xs = linspace(0, 2*pi, 60);

fprintf('\nplot with style strings (r--):\n');
plot(xs, sin(xs), 'r--');

title('sin and cos — style strings');
xlabel('x');
ylabel('y');
hold('on');
plot(xs, sin(xs), 'r--');
plot(xs, cos(xs), 'b.');
hold('off');
