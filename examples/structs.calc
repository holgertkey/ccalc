% structs.calc — Phase 13 (scalar structs, field access, struct built-ins)
%
% Topic: grouping related values into named fields, nested structs,
%        and built-in struct utilities.
%        All expected results are annotated for easy verification.
%
% Usage: ccalc structs.calc

% ── 1. Basic field assignment and read ──────────────────────────────────────

fprintf('=== 1. Basic field assignment ===\n')

pt.x = 3;
pt.y = 4;

fprintf('  pt.x     = %g       (exact: 3)\n',   pt.x)
fprintf('  pt.y     = %g       (exact: 4)\n',   pt.y)

dist = sqrt(pt.x^2 + pt.y^2);
fprintf('  dist     = %g       (exact: 5)\n',   dist)
fprintf('  isstruct = %d       (exact: 1)\n',   isstruct(pt))
fprintf('  isstruct(5) = %d    (exact: 0)\n',   isstruct(5))

% ── 2. struct() constructor ─────────────────────────────────────────────────

fprintf('\n=== 2. struct() constructor ===\n')

person = struct('name', 'Alice', 'age', 30, 'score', 98.5);

fprintf('  name  = %s          (exact: Alice)\n',  person.name)
fprintf('  age   = %g          (exact: 30)\n',     person.age)
fprintf('  score = %g          (exact: 98.5)\n',   person.score)

empty_s = struct();
fn = fieldnames(empty_s);
fprintf('  struct() fields = %d  (exact: 0)\n',   numel(fn))

% ── 3. Overwriting a field ───────────────────────────────────────────────────

fprintf('\n=== 3. Overwriting a field ===\n')

counter.n = 0;
fprintf('  n before = %g   (exact: 0)\n', counter.n)
counter.n = counter.n + 1;
counter.n = counter.n + 1;
counter.n = counter.n + 1;
fprintf('  n after  = %g   (exact: 3)\n', counter.n)

% ── 4. Nested structs ────────────────────────────────────────────────────────

fprintf('\n=== 4. Nested structs ===\n')

car.make   = 'Volvo';
car.engine.cylinders = 4;
car.engine.hp        = 190;
car.dims.length_m    = 4.76;
car.dims.width_m     = 1.85;

fprintf('  make             = %s    (exact: Volvo)\n', car.make)
fprintf('  engine.cylinders = %g   (exact: 4)\n',     car.engine.cylinders)
fprintf('  engine.hp        = %g   (exact: 190)\n',   car.engine.hp)
fprintf('  dims.length_m    = %g   (exact: 4.76)\n',  car.dims.length_m)

area = car.dims.length_m * car.dims.width_m;
fprintf('  footprint (m^2)  = %.4f (exact: 8.8060)\n', area)

% ── 5. fieldnames — get all field names ──────────────────────────────────────

fprintf('\n=== 5. fieldnames ===\n')

s.alpha = 1;
s.beta  = 2;
s.gamma = 3;

fn = fieldnames(s);
fprintf('  number of fields = %d   (exact: 3)\n', numel(fn))
for k = 1:numel(fn)
  fprintf('  fn{%d} = %s\n', k, fn{k})
end
% Expected:
%   fn{1} = alpha
%   fn{2} = beta
%   fn{3} = gamma

% ── 6. isfield — check for a field by name ───────────────────────────────────

fprintf('\n=== 6. isfield ===\n')

box.w = 10;
box.h = 5;
box.d = 3;

fprintf('  isfield(box, "w")   = %d   (exact: 1)\n', isfield(box, 'w'))
fprintf('  isfield(box, "d")   = %d   (exact: 1)\n', isfield(box, 'd'))
fprintf('  isfield(box, "vol") = %d   (exact: 0)\n', isfield(box, 'vol'))

% ── 7. rmfield — remove a field ──────────────────────────────────────────────

fprintf('\n=== 7. rmfield ===\n')

rec.id   = 42;
rec.temp = 101.3;
rec.flag = 0;

fprintf('  before: fields = %d   (exact: 3)\n', numel(fieldnames(rec)))
rec = rmfield(rec, 'flag');
fprintf('  after:  fields = %d   (exact: 2)\n', numel(fieldnames(rec)))
fprintf('  isfield flag   = %d   (exact: 0)\n', isfield(rec, 'flag'))
fprintf('  id still there = %d   (exact: 1)\n', isfield(rec, 'id'))

% ── 8. Structs in control flow ───────────────────────────────────────────────

fprintf('\n=== 8. Structs in control flow ===\n')

function show_grade(st)
  if st.score >= 90
    fprintf('  %s: A  (%g)\n', st.name, st.score)
  elseif st.score >= 75
    fprintf('  %s: B  (%g)\n', st.name, st.score)
  else
    fprintf('  %s: C  (%g)\n', st.name, st.score)
  end
end

show_grade(struct('name', 'Alice', 'score', 95))
show_grade(struct('name', 'Bob',   'score', 80))
show_grade(struct('name', 'Carl',  'score', 60))
% Expected:
%   Alice: A  (95)
%   Bob:   B  (80)
%   Carl:  C  (60)

% ── 9. Practical example — 3-D vector struct ─────────────────────────────────

fprintf('\n=== 9. 3-D vector operations ===\n')

function r = vec3(x, y, z)
  r.x = x;
  r.y = y;
  r.z = z;
end

function r = vec3_add(a, b)
  r = vec3(a.x + b.x, a.y + b.y, a.z + b.z);
end

function d = vec3_dot(a, b)
  d = a.x*b.x + a.y*b.y + a.z*b.z;
end

function n = vec3_norm(v)
  n = sqrt(v.x^2 + v.y^2 + v.z^2);
end

u = vec3(1, 0, 0);
v = vec3(0, 1, 0);
w = vec3(1, 2, 3);

s = vec3_add(u, v);
fprintf('  u + v = (%g, %g, %g)      (exact: 1, 1, 0)\n', s.x, s.y, s.z)
fprintf('  u . v = %g                (exact: 0)\n',        vec3_dot(u, v))
fprintf('  u . u = %g                (exact: 1)\n',        vec3_dot(u, u))
fprintf('  |w|   = %.4f              (exact: 3.7417)\n',   vec3_norm(w))

% Normalize w
n = vec3_norm(w);
w_hat = vec3(w.x/n, w.y/n, w.z/n);
fprintf('  w_hat . w_hat = %.6f      (exact: 1.000000)\n', vec3_dot(w_hat, w_hat))

fprintf('\nDone.\n')
