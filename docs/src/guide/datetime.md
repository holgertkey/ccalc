# Datetime & Duration

ccalc supports UTC datetime values and durations as first-class types.
All timestamps are stored internally as seconds since the Unix epoch (1970-01-01 00:00:00 UTC).

## Constructors

```
datetime('2024-06-01')               % from ISO 8601 date string
datetime('2024-06-01 09:30:00')      % date + time
datetime(2024, 6, 1)                 % year, month, day
datetime(2024, 6, 1, 9, 30, 0)      % year, month, day, hour, min, sec
datetime(ts, 'ConvertFrom', 'posixtime')  % from Unix timestamp scalar
```

`NaT` is the Not-a-Time constant, analogous to `NaN` for scalars.

## Duration constructors

```
duration(1, 30, 0)    % 1 hour 30 minutes → 5400 seconds
hours(2)              % 2 h  → Duration
minutes(90)           % 90 min → Duration
seconds(45)           % 45 s → Duration
days(1)               % 1 day → Duration
milliseconds(500)     % 500 ms → Duration
years(1)              % 365.2425 days → Duration
```

## Arithmetic

| Expression | Result type |
|---|---|
| `datetime + duration` | `DateTime` |
| `datetime - duration` | `DateTime` |
| `datetime - datetime` | `Duration` |
| `duration + duration` | `Duration` |
| `duration * scalar` | `Duration` |

```
t = datetime(2024, 1, 1);
d = hours(1);
t2 = t + d;           % 2024-01-01 01:00:00
elapsed = t2 - t;     % Duration: 01:00:00
```

## Component extractors

```
year(dt)     month(dt)    day(dt)
hour(dt)     minute(dt)   second(dt)
```

All extractors also work on `DateTimeArray`, returning a column vector.

## Duration extractors

```
hours(d)          % Duration → hours as scalar
minutes(d)        % Duration → minutes as scalar
seconds(d)        % Duration → seconds as scalar
days(d)           % Duration → days as scalar
milliseconds(d)   % Duration → milliseconds as scalar
```

## Predicates

```
isdatetime(x)    % 1 if x is DateTime or DateTimeArray
isduration(x)    % 1 if x is Duration or DurationArray
isnat(x)         % 1 if x is NaT (DateTime(NaN))
```

## Formatting and conversion

```
datestr(dt)                    % "15-Jan-2024 09:30:00"
datestr(dt, 'yyyy/MM/dd')      % custom pattern
datevec(dt)                    % [y m d H M S] row vector
datenum(dt)                    % MATLAB serial date number
datenum(y, m, d)               % MATLAB serial date from components
posixtime(dt)                  % Unix timestamp as scalar
```

### `datestr` pattern tokens

| Token | Description |
|---|---|
| `yyyy` | 4-digit year |
| `MMM` | 3-letter month abbreviation (Jan, Feb, …) |
| `MM` | 2-digit month |
| `dd` | 2-digit day |
| `HH` | 2-digit hour (24 h) |
| `mm` | 2-digit minute |
| `ss` | 2-digit second |
| `SSS` | 3-digit milliseconds |

## Array operations

Matrix literals build `DateTimeArray` or `DurationArray` when all elements are the same type:

```
t = [datetime(2024,1,1); datetime(2024,1,2); datetime(2024,1,3)];   % DateTimeArray
d = [hours(1); hours(2); hours(3)];                                  % DurationArray
```

`diff(arr)` computes successive differences:
- `DateTimeArray` → `DurationArray`
- `DurationArray` → `DurationArray`

```
t = [datetime(2024,1,1); datetime(2024,1,2); datetime(2024,1,3)];
d = diff(t);    % DurationArray of two 1-day durations
```

## `fprintf` and `sprintf`

`DateTime` and `Duration` values format as strings with `%s`:

```
dt  = datetime(2024, 6, 1);
dur = hours(2);
fprintf('%s\n', dt)    % 2024-06-01 00:00:00
fprintf('%s\n', dur)   % 02:00:00
s = sprintf('elapsed: %s', dur);
```
