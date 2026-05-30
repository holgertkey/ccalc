% dir_demo.m — directory listing with dir()
%
% Demonstrates MATLAB-compatible dir() function:
%   dir(path)      — list a directory
%   dir(pattern)   — list files matching a glob pattern
%   entries(i).name / .folder / .isdir / .bytes

% List the examples directory
entries = dir('examples');

fprintf('Total entries in examples/: %d\n', numel(entries));
fprintf('First entry: %s  isdir=%d\n', entries(1).name, entries(1).isdir);
fprintf('Second entry: %s  isdir=%d\n', entries(2).name, entries(2).isdir);

% Count files vs directories
n_dirs  = 0;
n_files = 0;
for k = 1:numel(entries)
    if entries(k).isdir
        n_dirs = n_dirs + 1;
    else
        n_files = n_files + 1;
    end
end
fprintf('Directories: %d  Files: %d\n', n_dirs, n_files);

% Glob pattern — list only .m files in current directory
m_files = dir('*.m');
fprintf('\n*.m files in current directory: %d\n', numel(m_files));
for k = 1:numel(m_files)
    fprintf('  %s  (%d bytes)\n', m_files(k).name, m_files(k).bytes);
end

% Non-existent path returns empty struct array (no error)
missing = dir('no_such_path_xyz');
fprintf('\ndir on missing path: %d entries\n', numel(missing));

% Access folder field — always an absolute path
entries2 = dir('examples/dir_demo');
fprintf('\nfolder field: %s\n', entries2(1).folder);
