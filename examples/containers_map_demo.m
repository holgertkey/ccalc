% containers.Map — key-value associative array demo

% Create a map from two cell arrays
m = containers.Map({'apple', 'banana', 'cherry'}, {1.5, 0.75, 2.0});
fprintf('Created map with %d entries\n', m.Count);

% Read a value by key
fprintf('apple costs: %.2f\n', m('apple'));

% Insert a new key
m('date') = 3.5;
fprintf('After insert: %d entries\n', m.Count);

% Update an existing key
m('banana') = 0.99;
fprintf('banana now costs: %.2f\n', m('banana'));

% Check key membership
fprintf('Has apple: %d\n', isKey(m, 'apple'));
fprintf('Has mango:  %d\n', isKey(m, 'mango'));

% List keys (sorted) and values (in key order)
k = keys(m);
v = values(m);
fprintf('Keys: ');
for i = 1:numel(k)
  fprintf('%s ', k{i});
end
fprintf('\n');

% Remove a key
remove(m, 'date');
fprintf('After remove: %d entries\n', m.Count);
