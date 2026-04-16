% euclid_helper.calc — Euclidean GCD
%
% Reads 'a' and 'b' from the caller's workspace, writes 'g' back.
% Do not run this file directly — source it with run() or source():
%
%   a = 252; b = 105;
%   run('euclid_helper')
%   % → g = 21

g = a;
r = b;
while r ~= 0
  temp = mod(g, r);
  g = r;
  r = temp;
end
