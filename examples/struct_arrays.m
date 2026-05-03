% struct_arrays.calc — Phase 13.5 (struct arrays, s(i).field, field collection)
%
% Topic: storing collections of labelled records, iterating over struct arrays,
%        collecting fields into vectors, and built-in utilities.
%        All expected results are annotated for easy verification.
%
% Usage: ccalc struct_arrays.calc

% ── 1. Creating a struct array by indexed assignment ─────────────────────────

fprintf('=== 1. Creating a struct array ===\n')

pts(1).x = 1;  pts(1).y = 0;
pts(2).x = 3;  pts(2).y = 4;
pts(3).x = 0;  pts(3).y = 5;

fprintf('  numel(pts)   = %d    (exact: 3)\n', numel(pts))
fprintf('  isstruct     = %d    (exact: 1)\n', isstruct(pts))
fprintf('  pts(2).x     = %g    (exact: 3)\n', pts(2).x)
fprintf('  pts(2).y     = %g    (exact: 4)\n', pts(2).y)

% ── 2. Accessing individual elements ─────────────────────────────────────────

fprintf('\n=== 2. Accessing elements ===\n')

p = pts(1);
fprintf('  pts(1).x = %g   (exact: 1)\n', p.x)
fprintf('  pts(1).y = %g   (exact: 0)\n', p.y)

% Direct chained access also works
fprintf('  pts(3).y = %g   (exact: 5)\n', pts(3).y)

% ── 3. Collecting a field across all elements ─────────────────────────────────

fprintf('\n=== 3. Field collection → matrix ===\n')

xs = pts.x;   % → 1×3 row vector [1 3 0]
ys = pts.y;   % → 1×3 row vector [0 4 5]

fprintf('  xs = [%g %g %g]   (exact: [1 3 0])\n', xs(1), xs(2), xs(3))
fprintf('  ys = [%g %g %g]   (exact: [0 4 5])\n', ys(1), ys(2), ys(3))

% Compute distances from origin for each point  (.^0.5 = element-wise sqrt)
dists = (xs .^ 2 + ys .^ 2) .^ 0.5;
fprintf('  dist(1) = %g      (exact: 1)\n',      dists(1))
fprintf('  dist(2) = %g      (exact: 5)\n',      dists(2))
fprintf('  dist(3) = %g      (exact: 5)\n',      dists(3))

% ── 4. Growing a struct array inside a loop ───────────────────────────────────

fprintf('\n=== 4. Building array in a loop ===\n')

clear data
for k = 1:5
  data(k).value = k * k;
  data(k).label = num2str(k);
end

fprintf('  numel(data) = %d   (exact: 5)\n', numel(data))
fprintf('  data(3).value = %g (exact: 9)\n',  data(3).value)
fprintf('  data(5).value = %g (exact: 25)\n', data(5).value)

vals = data.value;   % [1 4 9 16 25]
fprintf('  sum of squares = %g  (exact: 55)\n', sum(vals))

% ── 5. fieldnames / isfield on struct array ───────────────────────────────────

fprintf('\n=== 5. fieldnames and isfield ===\n')

fn = fieldnames(pts);
fprintf('  number of fields = %d    (exact: 2)\n', numel(fn))
fprintf('  fn{1} = %s               (exact: x)\n', fn{1})
fprintf('  fn{2} = %s               (exact: y)\n', fn{2})

fprintf('  isfield(pts,"x")  = %d  (exact: 1)\n', isfield(pts, 'x'))
fprintf('  isfield(pts,"z")  = %d  (exact: 0)\n', isfield(pts, 'z'))

% ── 6. String fields collected into a cell array ──────────────────────────────

fprintf('\n=== 6. String field collection → cell ===\n')

roster(1).name = 'Alice';  roster(1).score = 92;
roster(2).name = 'Bob';    roster(2).score = 78;
roster(3).name = 'Carol';  roster(3).score = 85;

names  = roster.name;    % → 1×3 cell (strings are not scalars)
scores = roster.score;   % → 1×3 matrix

fprintf('  names{1}  = %s   (exact: Alice)\n', names{1})
fprintf('  names{2}  = %s   (exact: Bob)\n',   names{2})
fprintf('  names{3}  = %s   (exact: Carol)\n', names{3})

avg = mean(scores);
fprintf('  avg score = %.4f  (exact: 85.0000)\n', avg)

% Print a simple leaderboard: find max score and print in order
fprintf('\n  Leaderboard:\n')
for k = 1:numel(scores)
  fprintf('    %s — %g\n', names{k}, scores(k))
end
% Expected: Alice (92), Bob (78), Carol (85)

% ── 7. Nested fields in a struct array ───────────────────────────────────────

fprintf('\n=== 7. Nested fields ===\n')

sensors(1).id = 1;  sensors(1).reading.temp = 22.5;  sensors(1).reading.hum = 55;
sensors(2).id = 2;  sensors(2).reading.temp = 24.1;  sensors(2).reading.hum = 60;
sensors(3).id = 3;  sensors(3).reading.temp = 19.8;  sensors(3).reading.hum = 48;

fprintf('  sensor 2 temp = %g   (exact: 24.1)\n', sensors(2).reading.temp)
fprintf('  sensor 3 hum  = %g   (exact: 48)\n',   sensors(3).reading.hum)

% ── 8. Practical example — inventory ledger ──────────────────────────────────

fprintf('\n=== 8. Inventory ledger ===\n')

items(1).name = 'Widget';    items(1).qty = 120;  items(1).price = 2.50;
items(2).name = 'Gadget';    items(2).qty =  45;  items(2).price = 9.99;
items(3).name = 'Doohickey'; items(3).qty = 300;  items(3).price = 0.75;

qtys   = items.qty;
prices = items.price;
values = qtys .* prices;   % element-wise: total value per item
total  = sum(values);

fprintf('  Widget    value = $%.2f  (exact: $300.00)\n', values(1))
fprintf('  Gadget    value = $%.2f   (exact: $449.55)\n', values(2))
fprintf('  Doohickey value = $%.2f  (exact: $225.00)\n', values(3))
fprintf('  Total inventory = $%.2f  (exact: $974.55)\n', total)

fprintf('\nDone.\n')
