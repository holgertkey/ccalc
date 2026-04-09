# File I/O

ccalc supports file I/O using MATLAB/Octave-compatible functions. You can read and write files using low-level file handles, load and save delimiter-separated data, query the filesystem, and persist workspace variables to named files.

---

## File handles

Open a file with `fopen`, write or read, then close with `fclose`:

```matlab
fd = fopen('log.txt', 'w');
fprintf(fd, 'result: %.4f\n', 3.14159);
fclose(fd);
```

Supported modes:

| Mode | Description |
|------|-------------|
| `'r'` | Read (file must exist) |
| `'w'` | Write — create or truncate |
| `'a'` | Append — create if missing |
| `'r+'` | Read and write |

`fopen` returns a file descriptor (integer ≥ 3) on success, or -1 on failure.

File descriptor 1 is stdout and 2 is stderr — they can be used with `fprintf` directly.

### Reading lines

```matlab
fd = fopen('data.txt', 'r');
line1 = fgetl(fd);    % strip trailing newline; returns -1 at EOF
line2 = fgetl(fd);
raw   = fgets(fd);    % keep trailing newline
fclose(fd);
```

### Closing all handles

```matlab
fclose('all');    % close every open file handle
```

---

## Delimiter-separated data

Write and read numeric matrices as CSV or TSV files:

```matlab
data = [0, 3.30, 0.012; 0.5, 3.28, 0.015; 1.0, 3.25, 0.018];

dlmwrite('measurements.csv', data);          % comma-separated (default)
dlmwrite('measurements.tsv', data, '\t');    % tab-separated

loaded = dlmread('measurements.csv');        % auto-detect delimiter
loaded = dlmread('measurements.tsv', '\t'); % explicit delimiter
```

`dlmread` returns a `Matrix`. All values in the file must be numeric; non-numeric data returns an error with the offending line number.

Auto-detection order: try `,` first, then `\t`, then whitespace.

---

## Filesystem queries

Check whether a file or directory exists before opening it:

```matlab
if isfile('data.csv')
    data = dlmread('data.csv');
end

isfolder('output/')    % 1 if directory exists, 0 otherwise

cwd = pwd()            % current working directory as a char array
```

`exist` checks variables or files:

```matlab
exist('x', 'var')      % 1 if variable x is in the workspace, 0 otherwise
exist('log.txt', 'file')  % 2 if file found, 0 otherwise
exist('x')             % checks workspace first, then filesystem
```

The numeric codes for `exist` match MATLAB: 1 = variable, 2 = file.

---

## Workspace with explicit path

Save and load workspace variables to a named file instead of the default path:

```matlab
R = 4700;
C = 220e-9;
label = 'RC filter';

save('session.mat');                    % save all to named file
save('session.mat', 'R', 'C');         % save specific variables only

clear R
clear C

load('session.mat');                    % load back

fprintf('R = %g\n', R)
```

The path argument can be a variable holding the path string:

```matlab
path = 'session.mat';
save(path);
load(path);
```

`save` and `load` without arguments use the default workspace path `~/.config/ccalc/workspace.toml` — the same as `ws` and `wl`.

### What gets saved

| Type | Persisted |
|------|-----------|
| Scalar | Yes |
| Char array (`'text'`) | Yes |
| String object (`"text"`) | Yes |
| Matrix | No |
| Complex | No |

---

## Example

The `examples/file_io.calc` file demonstrates all File I/O features end-to-end:

```bash
ccalc examples/file_io.calc
```

It covers: filesystem queries, writing to files with `fprintf`, line-by-line reading with `fgetl`/`fgets`, CSV/TSV with `dlmread`/`dlmwrite`, append mode, `save`/`load` with explicit paths and variable selection, and `fopen` error handling.
