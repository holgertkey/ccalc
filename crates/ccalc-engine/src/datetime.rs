/// Pure-Rust datetime arithmetic helpers (no external crate required).
///
/// All datetimes are UTC Unix timestamps (seconds since 1970-01-01 00:00:00 UTC).
/// The conversion algorithm is the one described by Howard Hinnant:
/// <https://howardhinnant.github.io/date_algorithms.html>
use std::time::{SystemTime, UNIX_EPOCH};

// ── Civil ↔ days ──────────────────────────────────────────────────────────────

/// Converts a proleptic Gregorian date (y, m, d) to a day count since the Unix epoch.
/// Month is 1-based; day is 1-based.
pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era: i64 = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = (y - era * 400) as u64;
    let doy = (153 * (if m > 2 { m as u64 - 3 } else { m as u64 + 9 }) + 2) / 5 + d as u64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe as i64 - 719468
}

/// Converts a day count since the Unix epoch to a proleptic Gregorian date.
/// Returns `(year, month, day)` where month and day are 1-based.
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

// ── Timestamp ↔ components ────────────────────────────────────────────────────

/// Decomposes a Unix timestamp (seconds) into calendar components.
/// Returns `(year, month, day, hour, minute, second_f64)`.
pub fn timestamp_to_civil(ts: f64) -> (i64, u32, u32, u32, u32, f64) {
    let ts_i = ts.floor() as i64;
    let sub_sec = ts - ts.floor();
    let sod = ts_i.rem_euclid(86400) as u32;
    let days = (ts_i - sod as i64) / 86400;
    let h = sod / 3600;
    let m = (sod % 3600) / 60;
    let s = (sod % 60) as f64 + sub_sec;
    let (y, mo, d) = civil_from_days(days);
    (y, mo, d, h, m, s)
}

/// Assembles a Unix timestamp (seconds) from calendar components.
pub fn civil_to_timestamp(y: i64, mo: u32, d: u32, h: u32, mi: u32, s: f64) -> f64 {
    let days = days_from_civil(y, mo, d);
    days as f64 * 86400.0 + h as f64 * 3600.0 + mi as f64 * 60.0 + s
}

// ── Parsing ────────────────────────────────────────────────────────────────────

/// Parses an ISO 8601 date string: `yyyy-MM-dd` or `yyyy-MM-dd HH:mm:ss`.
pub fn parse_iso8601(s: &str) -> Result<f64, String> {
    let s = s.trim();
    if s.len() == 10 {
        // yyyy-MM-dd
        let y = parse_component(s, 0, 4)?;
        let mo = parse_component(s, 5, 7)?;
        let d = parse_component(s, 8, 10)?;
        if s.as_bytes().get(4) != Some(&b'-') || s.as_bytes().get(7) != Some(&b'-') {
            return Err(format!("Invalid date format: '{s}'"));
        }
        let ts = civil_to_timestamp(y as i64, mo, d, 0, 0, 0.0);
        return Ok(ts);
    }
    if s.len() == 19 {
        // yyyy-MM-dd HH:mm:ss
        let y = parse_component(s, 0, 4)?;
        let mo = parse_component(s, 5, 7)?;
        let d = parse_component(s, 8, 10)?;
        let h = parse_component(s, 11, 13)?;
        let mi = parse_component(s, 14, 16)?;
        let sec = parse_component(s, 17, 19)?;
        let sep_ok = s.as_bytes().get(4) == Some(&b'-')
            && s.as_bytes().get(7) == Some(&b'-')
            && (s.as_bytes().get(10) == Some(&b' ') || s.as_bytes().get(10) == Some(&b'T'))
            && s.as_bytes().get(13) == Some(&b':')
            && s.as_bytes().get(16) == Some(&b':');
        if !sep_ok {
            return Err(format!("Invalid datetime format: '{s}'"));
        }
        let ts = civil_to_timestamp(y as i64, mo, d, h, mi, sec as f64);
        return Ok(ts);
    }
    Err(format!("Unsupported datetime format: '{s}'"))
}

fn parse_component(s: &str, start: usize, end: usize) -> Result<u32, String> {
    s.get(start..end)
        .and_then(|t| t.parse::<u32>().ok())
        .ok_or_else(|| format!("Invalid numeric component in '{s}'"))
}

// ── Formatting ─────────────────────────────────────────────────────────────────

/// Formats a Unix timestamp as `yyyy-MM-dd HH:mm:ss`.
pub fn format_datetime(ts: f64) -> String {
    if ts.is_nan() {
        return "NaT".to_string();
    }
    let (y, mo, d, h, mi, s) = timestamp_to_civil(ts);
    let sec_i = s.floor() as u32;
    let sub = s - s.floor();
    if sub > 1e-9 {
        let ms = (sub * 1000.0).round() as u32;
        format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{sec_i:02}.{ms:03}")
    } else {
        format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{sec_i:02}")
    }
}

/// Formats a duration in seconds as `HH:MM:SS` or `Nd HH:MM:SS`.
/// Adds `.mmm` suffix when sub-second precision is present.
pub fn format_duration(secs: f64) -> String {
    let abs = secs.abs();
    let sign = if secs < 0.0 { "-" } else { "" };
    let total_sec = abs.floor() as u64;
    let sub = abs - abs.floor();
    let h = total_sec / 3600;
    let m = (total_sec % 3600) / 60;
    let s = total_sec % 60;
    let ms_suffix = if sub > 1e-9 {
        format!(".{:03}", (sub * 1000.0).round() as u32)
    } else {
        String::new()
    };
    if h >= 24 {
        let days = h / 24;
        let rem_h = h % 24;
        format!("{sign}{days}d {rem_h:02}:{m:02}:{s:02}{ms_suffix}")
    } else {
        format!("{sign}{h:02}:{m:02}:{s:02}{ms_suffix}")
    }
}

const MONTH_ABBR: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Formats a Unix timestamp using a pattern string.
/// Supported tokens: `yyyy`, `MMM` (abbreviated month name), `MM`, `dd`, `HH`, `mm`, `ss`, `SSS`.
pub fn format_datestr(ts: f64, pattern: &str) -> String {
    if ts.is_nan() {
        return "NaT".to_string();
    }
    let (y, mo, d, h, mi, s) = timestamp_to_civil(ts);
    let sec_i = s.floor() as u32;
    let ms = (s.fract() * 1000.0).round() as u32;
    let mo_abbr = MONTH_ABBR
        .get(mo.saturating_sub(1) as usize)
        .copied()
        .unwrap_or("???");
    // Replace MMM before MM so the longer token wins.
    let result = pattern
        .replace("yyyy", &format!("{y:04}"))
        .replace("MMM", mo_abbr)
        .replace("MM", &format!("{mo:02}"))
        .replace("dd", &format!("{d:02}"))
        .replace("HH", &format!("{h:02}"))
        .replace("mm", &format!("{mi:02}"))
        .replace("ss", &format!("{sec_i:02}"))
        .replace("SSS", &format!("{ms:03}"));
    result
}

/// Returns the current UTC time as a Unix timestamp.
pub fn now_timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// Returns midnight today as a Unix timestamp.
pub fn today_timestamp() -> f64 {
    let ts = now_timestamp();
    let days = (ts / 86400.0).floor();
    days * 86400.0
}

// ── MATLAB datenum conversion ──────────────────────────────────────────────────

/// MATLAB serial date number: days since 0000-Jan-00 (= 0000-Dec-31).
/// Offset from Unix epoch: 1970-01-01 = datenum 719529.
const MATLAB_EPOCH_DAYS: f64 = 719529.0;

/// Converts a Unix timestamp to a MATLAB serial date number.
pub fn to_datenum(ts: f64) -> f64 {
    ts / 86400.0 + MATLAB_EPOCH_DAYS
}

/// Converts a MATLAB serial date number to a Unix timestamp.
pub fn from_datenum(dn: f64) -> f64 {
    (dn - MATLAB_EPOCH_DAYS) * 86400.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoch_roundtrip() {
        // Unix epoch = 1970-01-01 00:00:00
        let (y, mo, d, h, mi, s) = timestamp_to_civil(0.0);
        assert_eq!((y, mo, d, h, mi), (1970, 1, 1, 0, 0));
        assert!((s - 0.0).abs() < 1e-9);
        let ts = civil_to_timestamp(1970, 1, 1, 0, 0, 0.0);
        assert!((ts - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_known_date() {
        // 2024-01-15 09:30:00 UTC
        let ts = civil_to_timestamp(2024, 1, 15, 9, 30, 0.0);
        let (y, mo, d, h, mi, s) = timestamp_to_civil(ts);
        assert_eq!((y, mo, d, h, mi), (2024, 1, 15, 9, 30));
        assert!((s - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_leap_year_boundary() {
        // 2024 is a leap year; Feb 29 should exist
        let ts = civil_to_timestamp(2024, 2, 29, 0, 0, 0.0);
        let (y, mo, d, ..) = timestamp_to_civil(ts);
        assert_eq!((y, mo, d), (2024, 2, 29));
        // Day after leap day = March 1
        let ts2 = ts + 86400.0;
        let (y2, mo2, d2, ..) = timestamp_to_civil(ts2);
        assert_eq!((y2, mo2, d2), (2024, 3, 1));
    }

    #[test]
    fn test_iso8601_parsing() {
        let ts = parse_iso8601("2024-01-15").unwrap();
        let (y, mo, d, h, mi, _) = timestamp_to_civil(ts);
        assert_eq!((y, mo, d, h, mi), (2024, 1, 15, 0, 0));

        let ts2 = parse_iso8601("2024-01-15 09:30:00").unwrap();
        let (y2, mo2, d2, h2, mi2, _) = timestamp_to_civil(ts2);
        assert_eq!((y2, mo2, d2, h2, mi2), (2024, 1, 15, 9, 30));
    }

    #[test]
    fn test_format_datetime() {
        let ts = civil_to_timestamp(2024, 1, 15, 9, 30, 0.0);
        assert_eq!(format_datetime(ts), "2024-01-15 09:30:00");
        assert_eq!(format_datetime(f64::NAN), "NaT");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(3600.0), "01:00:00");
        assert_eq!(format_duration(90.0), "00:01:30");
        assert_eq!(format_duration(86400.0 + 3600.0), "1d 01:00:00");
        assert_eq!(format_duration(0.5), "00:00:00.500");
    }

    #[test]
    fn test_datenum_roundtrip() {
        // 1970-01-01 = datenum 719529
        assert!((to_datenum(0.0) - 719529.0).abs() < 1e-9);
        assert!((from_datenum(719529.0) - 0.0).abs() < 1e-9);
    }
}
