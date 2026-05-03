% datetime.m — Phase 22 feature demo
%
% Covers: datetime constructors, NaT, duration constructors,
%         arithmetic, component extractors, predicates,
%         formatting (datestr, datevec, datenum, posixtime),
%         and duration decomposition.
%
% Usage:
%   ccalc examples/datetime.m
%   cargo run -- examples/datetime.m

% ── 1. Constructors ──────────────────────────────────────────────────────────

fprintf('=== 1. Constructors ===\n')

% From ISO 8601 date string
d1 = datetime('2024-06-01');
fprintf('datetime(''2024-06-01'')              → %s\n', datestr(d1))

% From ISO 8601 date+time string
d2 = datetime('2024-06-01 09:30:00');
fprintf('datetime(''2024-06-01 09:30:00'')     → %s\n', datestr(d2))

% From year, month, day components
d3 = datetime(2024, 6, 1);
fprintf('datetime(2024, 6, 1)                 → %s\n', datestr(d3))

% From year, month, day, hour, minute, second
d4 = datetime(2024, 6, 1, 9, 30, 0);
fprintf('datetime(2024, 6, 1, 9, 30, 0)       → %s\n', datestr(d4))

% From Unix timestamp (posixtime)
ts = 1717200000;   % 2024-06-01 00:00:00 UTC
d5 = datetime(ts, 'ConvertFrom', 'posixtime');
fprintf('datetime(%g, ''ConvertFrom'', ''posixtime'') → %s\n', ts, datestr(d5))

% NaT — Not-a-Time sentinel (analogous to NaN for numbers)
fprintf('\nNaT is the Not-a-Time constant:\n')
fprintf('  isnat(NaT)  → %d  (1)\n', isnat(NaT))
fprintf('  isnat(d1)   → %d  (0)\n', isnat(d1))

% ── 2. Duration constructors ─────────────────────────────────────────────────

fprintf('\n=== 2. Duration constructors ===\n')
fprintf('Durations display as HH:MM:SS\n\n')

% Unbracketed assignments display via the REPL formatter
dur_hms  = duration(1, 30, 0)     % 1 h 30 min → 01:30:00
dur_h    = hours(2)               % 02:00:00
dur_min  = minutes(90)            % 01:30:00
dur_sec  = seconds(45)            % 00:00:45
dur_day  = days(1)                % 24:00:00
dur_ms   = milliseconds(500)      % 00:00:00.500
dur_yr   = years(1)               % ~8765.82 h

% ── 3. Arithmetic ────────────────────────────────────────────────────────────

fprintf('\n=== 3. Arithmetic ===\n')

t  = datetime(2024, 1, 1);
h1 = hours(1);

% datetime + duration → DateTime
t2 = t + h1;
fprintf('datetime(2024,1,1) + hours(1)      → %s\n', datestr(t2))

% datetime - duration → DateTime
t3 = t2 - minutes(30);
fprintf('t2 - minutes(30)                   → %s\n', datestr(t3))

% datetime - datetime → Duration (displayed as HH:MM:SS)
elapsed = t2 - t
fprintf('  → %g minutes elapsed\n', minutes(elapsed))

% duration + duration → Duration
total = hours(2) + minutes(15)
fprintf('  → %g hours total\n', hours(total))

% duration * scalar → Duration
stretch = hours(3) * 2
fprintf('  → %g hours\n', hours(stretch))

% ── 4. Component extractors ──────────────────────────────────────────────────

fprintf('\n=== 4. Component extractors ===\n')

ref = datetime(2024, 7, 4, 13, 45, 30);
fprintf('Reference: %s\n', datestr(ref))
fprintf('  year   → %g\n', year(ref))
fprintf('  month  → %g\n', month(ref))
fprintf('  day    → %g\n', day(ref))
fprintf('  hour   → %g\n', hour(ref))
fprintf('  minute → %g\n', minute(ref))
fprintf('  second → %g\n', second(ref))

% Duration extractors — numeric values from a Duration value
d = duration(2, 30, 15);   % 2 h 30 m 15 s
fprintf('\nDuration 02:30:15 decomposed:\n')
fprintf('  hours(d)        → %g\n', hours(d))
fprintf('  minutes(d)      → %g\n', minutes(d))
fprintf('  seconds(d)      → %g\n', seconds(d))
fprintf('  milliseconds(d) → %g\n', milliseconds(d))
fprintf('  days(d)         → %.6f\n', days(d))

% ── 5. Predicates ────────────────────────────────────────────────────────────

fprintf('\n=== 5. Predicates ===\n')

t6   = datetime(2025, 3, 14);
dur8 = hours(1);

fprintf('isdatetime(t6)    → %d  (1)\n', isdatetime(t6))
fprintf('isdatetime(dur8)  → %d  (0)\n', isdatetime(dur8))
fprintf('isduration(dur8)  → %d  (1)\n', isduration(dur8))
fprintf('isduration(t6)    → %d  (0)\n', isduration(t6))
fprintf('isnat(NaT)        → %d  (1)\n', isnat(NaT))
fprintf('isnat(t6)         → %d  (0)\n', isnat(t6))

% ── 6. Formatting and conversion ─────────────────────────────────────────────

fprintf('\n=== 6. Formatting and conversion ===\n')

dt = datetime(2024, 3, 15, 9, 30, 0);
fprintf('Reference: %s\n\n', datestr(dt))

% datestr — default and custom patterns
fprintf('datestr(dt)                      → %s\n', datestr(dt))
fprintf('datestr(dt, ''yyyy/MM/dd'')       → %s\n', datestr(dt, 'yyyy/MM/dd'))
fprintf('datestr(dt, ''dd-MMM-yyyy'')      → %s\n', datestr(dt, 'dd-MMM-yyyy'))
fprintf('datestr(dt, ''HH:mm:ss'')         → %s\n', datestr(dt, 'HH:mm:ss'))
fprintf('datestr(dt, ''yyyy-MM-dd HH:mm'') → %s\n', datestr(dt, 'yyyy-MM-dd HH:mm'))

% datevec — [y m d H M S] row vector
v = datevec(dt);
fprintf('\ndatevec(dt)  → [%g %g %g %g %g %g]\n', v(1), v(2), v(3), v(4), v(5), v(6))

% datenum — MATLAB serial date number (days since 0000-01-00)
dn = datenum(dt);
fprintf('datenum(dt)             → %g\n', dn)
fprintf('datenum(2024, 3, 15)    → %g\n', datenum(2024, 3, 15))

% posixtime — Unix timestamp in seconds
pt = posixtime(dt);
fprintf('posixtime(dt)           → %g\n', pt)

% Round-trip: posixtime → datetime → posixtime
rt = datetime(pt, 'ConvertFrom', 'posixtime');
fprintf('round-trip via posixtime  → %s\n', datestr(rt))

% ── 7. Practical — project timeline ─────────────────────────────────────────

fprintf('\n=== 7. Practical — project timeline ===\n')

kickoff = datetime(2024, 9, 2);
alpha   = datetime(2024, 10, 14);
beta    = datetime(2024, 11, 25);
release = datetime(2025, 1, 20);

fprintf('Milestones:\n')
fprintf('  Kickoff   %s\n', datestr(kickoff, 'dd-MMM-yyyy'))
fprintf('  Alpha     %s\n', datestr(alpha,   'dd-MMM-yyyy'))
fprintf('  Beta      %s\n', datestr(beta,    'dd-MMM-yyyy'))
fprintf('  Release   %s\n', datestr(release, 'dd-MMM-yyyy'))

% Sprint durations via datetime - datetime → Duration, then days()
sprint1 = days(alpha   - kickoff);
sprint2 = days(beta    - alpha);
sprint3 = days(release - beta);
total   = days(release - kickoff);

fprintf('\nSprint durations:\n')
fprintf('  Kickoff → Alpha   %g days\n', sprint1)
fprintf('  Alpha   → Beta    %g days\n', sprint2)
fprintf('  Beta    → Release %g days\n', sprint3)
fprintf('\nTotal project duration: %g days\n', total)

% Find the release day of week using datestr weekday abbrev
fprintf('\nRelease falls on: %s\n', datestr(release, 'dd-MMM-yyyy'))
fprintf('  year %g, month %g, day %g\n', year(release), month(release), day(release))

fprintf('\nDone.\n')
