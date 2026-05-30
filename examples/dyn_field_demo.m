% dyn_field_demo.calc — Phase 33c: dynamic struct field access s.(fname)
%
% Demonstrates reading and writing struct fields whose names are determined
% at runtime from string variables or expressions.
%
% Usage: ccalc examples/dyn_field_demo/dyn_field_demo.calc

% ── 1. Basic read and write ───────────────────────────────────────────────────

fprintf('=== 1. Basic read / write ===\n')

planet.name   = 'Earth';
planet.mass   = 5.972e24;
planet.radius = 6.371e6;

% Read a field whose name is stored in a variable
field = 'mass';
fprintf('  planet.(%s) = %g kg\n', field, planet.(field))

field = 'radius';
fprintf('  planet.(%s) = %g m\n', field, planet.(field))

% Write a new field dynamically
new_field = 'moons';
planet.(new_field) = 1;
fprintf('  planet.(%s) = %d\n', new_field, planet.(new_field))

% Expected output:
%   planet.(mass)   = 5.972e+24 kg
%   planet.(radius) = 6.371e+06 m
%   planet.(moons)  = 1

% ── 2. Iterate over a list of field names ─────────────────────────────────────

fprintf('\n=== 2. Loop over field names ===\n')

stats.min  = -3.14;
stats.max  =  9.81;
stats.mean =  2.71;
stats.std  =  1.41;

fields = {'min', 'max', 'mean', 'std'};
for k = 1:numel(fields)
  fname = fields{k};
  fprintf('  stats.%-6s = %6.2f\n', fname, stats.(fname))
end

% Expected output:
%   stats.min    =  -3.14
%   stats.max    =   9.81
%   stats.mean   =   2.71
%   stats.std    =   1.41

% ── 3. Build a struct from parallel name/value arrays ─────────────────────────

fprintf('\n=== 3. Build struct from name/value arrays ===\n')

keys   = {'x', 'y', 'z'};
values = {10,  20,  30};

point = struct();
for k = 1:numel(keys)
  point.(keys{k}) = values{k};
end

fprintf('  point.x = %d\n', point.x)
fprintf('  point.y = %d\n', point.y)
fprintf('  point.z = %d\n', point.z)

% Expected output:
%   point.x = 10
%   point.y = 20
%   point.z = 30

% ── 4. Compute a field name from a prefix and index ───────────────────────────

fprintf('\n=== 4. Computed field names ===\n')

sensor = struct();
for k = 1:4
  fname = strcat('ch', num2str(k));
  sensor.(fname) = k * 0.5;
end

for k = 1:4
  fname = strcat('ch', num2str(k));
  fprintf('  sensor.%s = %.1f\n', fname, sensor.(fname))
end

% Expected output:
%   sensor.ch1 = 0.5
%   sensor.ch2 = 1.0
%   sensor.ch3 = 1.5
%   sensor.ch4 = 2.0

% ── 5. Inline string literal as field name ────────────────────────────────────

fprintf('\n=== 5. Literal string in .(''...'') ===\n')

cfg.width  = 1920;
cfg.height = 1080;

% The field name can be any expression that evaluates to a string,
% including an inline literal.
fprintf('  width  = %d\n', cfg.('width'))
fprintf('  height = %d\n', cfg.('height'))

% Expected output:
%   width  = 1920
%   height = 1080

fprintf('\ndyn_field_demo complete.\n')
