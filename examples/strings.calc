% strings.calc — Phase 9 feature demo
%
% Covers: char arrays ('single-quoted'), string objects ("double-quoted"),
%         char arithmetic, comparison, and string built-ins.
%
% Usage: ccalc strings.calc

% --- 1. Creating strings ---

fprintf('=== 1. Creating strings ===\n')

% Single-quoted: char array (MATLAB classic style)
greeting = 'Hello, World!';
fprintf('greeting = ')
disp(greeting)

% Double-quoted: string object (modern style)
note = "A string object";
fprintf('note     = ')
disp(note)

% Escaped single quote inside a char array: '' doubles the quote
phrase = 'it''s a test';
fprintf("phrase   = ")
disp(phrase)

% Escape sequences inside double-quoted strings
tab_demo = "col1\tcol2";
fprintf('tab_demo = ')
disp(tab_demo)

% --- 2. String length and size ---

fprintf('\n=== 2. Length and size ===\n')

s = 'abcde';
fprintf('s = ')
disp(s)
fprintf('length(s)  =>  %d\n', length(s))
fprintf('numel(s)   =>  %d\n', numel(s))
fprintf('size(s)    =>  ')
disp(size(s))

% String object is a scalar (1x1)
t = "hello";
fprintf('\nt = "hello"\n')
fprintf('length(t)  =>  %d\n', length(t))
fprintf('size(t)    =>  ')
disp(size(t))

% --- 3. Type checks ---

fprintf('\n=== 3. Type checks ===\n')

fprintf("ischar('hello')   =>  %d\n", ischar('hello'))
fprintf('isstring("hello")  =>  %d\n', isstring("hello"))
fprintf('ischar("hello")    =>  %d\n', ischar("hello"))
fprintf('ischar(42)         =>  %d\n', ischar(42))

% --- 4. Char array arithmetic ---

fprintf('\n=== 4. Char array arithmetic ===\n')
fprintf('Char arrays convert to ASCII codes before arithmetic.\n\n')

% Single char to ASCII code
fprintf("'A' + 0  =>  %d\n", 'A' + 0)

% Shift a letter
fprintf("'a' + 1  =>  %d\n", 'a' + 1)

% Multi-char: produces a numeric row vector
fprintf("'abc' + 0  =>  ")
disp('abc' + 0)

% Increment every character by 1
fprintf("'abc' + 1  =>  ")
disp('abc' + 1)

% ASCII codes for 'Hello'
fprintf("'Hello' + 0  =>  ")
disp('Hello' + 0)

% Char comparison: element-wise, returns 0/1 vector
fprintf("'abc' == 'aXc'  =>  ")
disp('abc' == 'aXc')

% --- 5. String object concatenation ---

fprintf('\n=== 5. String object concatenation ===\n')
fprintf('Use + to concatenate string objects.\n\n')

first = "Hello";
last  = " World";
full  = first + last;
fprintf('first + last  =>  ')
disp(full)

% strcat works on both char arrays and string objects
fprintf("strcat('foo', 'bar')  =>  ")
disp(strcat('foo', 'bar'))
fprintf('strcat("foo", "bar")   =>  ')
disp(strcat("foo", "bar"))

% --- 6. Comparison functions ---

fprintf('\n=== 6. Comparison functions ===\n')

fprintf("strcmp('abc', 'abc')   =>  %d\n", strcmp('abc', 'abc'))
fprintf("strcmp('abc', 'ABC')   =>  %d\n", strcmp('abc', 'ABC'))
fprintf("strcmpi('abc', 'ABC')  =>  %d\n", strcmpi('abc', 'ABC'))

% String objects: use == operator
fprintf('"hello" == "hello"   =>  %d\n', "hello" == "hello")
fprintf('"hello" == "world"   =>  %d\n', "hello" == "world")

% --- 7. Case conversion and trimming ---

fprintf('\n=== 7. Case and trim ===\n')

fprintf("upper('hello')        =>  ")
disp(upper('hello'))
fprintf("lower('WORLD')        =>  ")
disp(lower('WORLD'))
fprintf("strtrim('  spaces  ') =>  ")
disp(strtrim('  spaces  '))
fprintf('strtrim("  trim me  ") =>  ')
disp(strtrim("  trim me  "))

% --- 8. Search and replace ---

fprintf('\n=== 8. strrep ===\n')

src = 'the quick brown fox';
fprintf('src = ')
disp(src)
fprintf("strrep(src, 'fox', 'cat')    =>  ")
disp(strrep(src, 'fox', 'cat'))
fprintf("strrep(src, 'quick ', '')   =>  ")
disp(strrep(src, 'quick ', ''))

% --- 9. Number conversions ---

fprintf('\n=== 9. Number conversions ===\n')

fprintf('num2str(42)         =>  ')
disp(num2str(42))
fprintf('num2str(3.14159)    =>  ')
disp(num2str(3.14159))
fprintf('num2str(3.14159, 2) =>  ')
disp(num2str(3.14159, 2))

fprintf("str2double('2.718')  =>  %.4f\n", str2double('2.718'))
fprintf("str2double('abc')   =>  ")
disp(str2double('abc'))

fprintf("str2num('100')      =>  %d\n", str2num('100'))

% --- 10. sprintf for string building ---

fprintf('\n=== 10. sprintf ===\n')

msg = sprintf('Line one\nLine two\n');
fprintf('sprintf result:\n')
disp(msg)

tab_str = sprintf('A\tB\tC\n');
fprintf('With tabs:\n')
disp(tab_str)

% --- 11. Practical: unit label formatting ---

fprintf('\n=== 11. Practical — unit labels ===\n')

R = 4700;
C = 2.2e-9;
f0 = 1 / (2 * pi * R * C);

fprintf('RC low-pass filter\n')
fprintf('  R  = %d Ohm\n', R)
fprintf('  C  = %.1f nF\n', C * 1e9)
fprintf('  f0 = %.2f Hz\n', f0)
