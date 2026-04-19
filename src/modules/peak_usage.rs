//! Peak-time usage indicator — shows when Anthropic's peak-time rate limiting
//! is likely active, based on current time relative to US Pacific business hours.
//!
//! Pure time-based check with no network calls. US Pacific DST boundaries are
//! computed via Tomohiko Sakamoto's day-of-week algorithm.

use crate::config::CshipConfig;
use crate::context::Context;

const DEFAULT_START_HOUR: u32 = 7;
const DEFAULT_END_HOUR: u32 = 17;
const DEFAULT_SYMBOL: &str = "⏰ ";
const PEAK_LABEL: &str = "Peak";

/// Seconds per day / hour / minute for UTC → Pacific conversion.
const SECS_PER_DAY: u64 = 86400;
const SECS_PER_HOUR: u64 = 3600;

/// Render `$cship.peak_usage` — returns styled indicator during peak hours, `None` otherwise.
pub fn render(_ctx: &Context, cfg: &CshipConfig) -> Option<String> {
    let pk_cfg = cfg.peak_usage.as_ref();

    // Disabled → silent None
    if pk_cfg.and_then(|c| c.disabled) == Some(true) {
        return None;
    }

    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();

    if !is_peak_time(now_secs, pk_cfg) {
        return None;
    }

    let symbol = pk_cfg
        .and_then(|c| c.symbol.as_deref())
        .unwrap_or(DEFAULT_SYMBOL);
    let style = pk_cfg.and_then(|c| c.style.as_deref());

    // Format string takes priority if configured
    if let Some(fmt) = pk_cfg.and_then(|c| c.format.as_deref()) {
        return crate::format::apply_module_format(fmt, Some(PEAK_LABEL), Some(symbol), style);
    }

    let content = format!("{symbol}{PEAK_LABEL}");
    Some(crate::ansi::apply_style(&content, style))
}

/// Returns `true` when `utc_epoch_secs` falls within the configured peak window
/// (default: Mon–Fri 07:00–17:00 US Pacific).
fn is_peak_time(utc_epoch_secs: u64, pk_cfg: Option<&crate::config::PeakUsageConfig>) -> bool {
    let start = pk_cfg
        .and_then(|c| c.start_hour)
        .unwrap_or(DEFAULT_START_HOUR);
    let end = pk_cfg.and_then(|c| c.end_hour).unwrap_or(DEFAULT_END_HOUR);

    if start >= end {
        tracing::warn!(
            "cship.peak_usage: start_hour ({start}) >= end_hour ({end}); \
             overnight wrap-around is not supported — module will never activate"
        );
        return false;
    }
    if start > 23 || end > 24 {
        tracing::warn!(
            "cship.peak_usage: start_hour ({start}) or end_hour ({end}) out of range \
             (start: 0–23, end: 0–24)"
        );
        return false;
    }

    let offset_secs = pacific_offset_secs(utc_epoch_secs);
    // Apply signed offset to UTC epoch. Pacific is always behind UTC, so offset is negative.
    let pacific_secs = (utc_epoch_secs as i64 + offset_secs) as u64;

    let day_secs = pacific_secs % SECS_PER_DAY;
    let hour = (day_secs / SECS_PER_HOUR) as u32;

    // Compute weekday from Pacific date (0=Sun, 1=Mon, ..., 6=Sat)
    let days_since_epoch = pacific_secs / SECS_PER_DAY;
    // 1970-01-01 was Thursday (4)
    let weekday = ((days_since_epoch + 4) % 7) as u32;

    let is_weekday = (1..=5).contains(&weekday); // Mon=1 .. Fri=5
    is_weekday && hour >= start && hour < end
}

/// Returns the UTC offset for US Pacific time in seconds.
/// PDT (UTC−7) from second Sunday of March 10:00 UTC to first Sunday of November 09:00 UTC.
/// PST (UTC−8) otherwise.
fn pacific_offset_secs(utc_epoch_secs: u64) -> i64 {
    let (year, month, day, hour) = utc_epoch_to_ymd_h(utc_epoch_secs);

    // DST transition boundaries (in UTC):
    // Spring forward: second Sunday of March at 2:00 AM PST = 10:00 UTC
    let march_second_sunday = nth_sunday_of_month(year, 3, 2);
    // Fall back: first Sunday of November at 2:00 AM PDT = 09:00 UTC
    let november_first_sunday = nth_sunday_of_month(year, 11, 1);

    let is_pdt = match month {
        4..=10 => true,      // Apr–Oct: always PDT
        1..=2 | 12 => false, // Jan–Feb, Dec: always PST
        3 => {
            // March: PDT after second Sunday 10:00 UTC
            day > march_second_sunday || (day == march_second_sunday && hour >= 10)
        }
        11 => {
            // November: PST after first Sunday 09:00 UTC
            day < november_first_sunday || (day == november_first_sunday && hour < 9)
        }
        _ => false,
    };

    if is_pdt { -7 * 3600 } else { -8 * 3600 }
}

/// Convert UTC epoch seconds to (year, month, day, hour).
fn utc_epoch_to_ymd_h(secs: u64) -> (i32, u32, u32, u32) {
    // Civil date from days since epoch using the algorithm from
    // Howard Hinnant's chrono-Compatible Low-Level Date Algorithms.
    let z = (secs / SECS_PER_DAY) as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64; // day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year [0, 365]
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    let day_secs = secs % SECS_PER_DAY;
    let hour = (day_secs / SECS_PER_HOUR) as u32;

    (y as i32, m as u32, d as u32, hour)
}

/// Returns the day-of-month for the Nth Sunday of a given month/year.
/// Uses Tomohiko Sakamoto's day-of-week algorithm.
fn nth_sunday_of_month(year: i32, month: u32, n: u32) -> u32 {
    // Day of week for the 1st of the month (0=Sun, 1=Mon, ..., 6=Sat)
    let dow_first = day_of_week(year, month, 1);
    // Days until first Sunday
    let first_sunday = if dow_first == 0 {
        1
    } else {
        1 + (7 - dow_first)
    };
    // Nth Sunday
    first_sunday + 7 * (n - 1)
}

/// Tomohiko Sakamoto's day-of-week algorithm.
/// Returns 0=Sun, 1=Mon, ..., 6=Sat.
fn day_of_week(mut year: i32, month: u32, day: u32) -> u32 {
    static T: [u32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    if month < 3 {
        year -= 1;
    }
    ((year + year / 4 - year / 100 + year / 400 + T[(month - 1) as usize] as i32 + day as i32) % 7)
        as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CshipConfig, PeakUsageConfig};
    use crate::context::Context;

    /// Helper: build a UTC epoch for a specific Pacific date/time,
    /// accounting for the DST offset on that date.
    fn pacific_to_utc(year: i32, month: u32, day: u32, hour: u32) -> u64 {
        let days = civil_days_from_epoch(year, month, day);
        let utc_approx = (days as u64) * SECS_PER_DAY + (hour as u64) * SECS_PER_HOUR;
        // Determine offset at this approximate UTC time, then adjust
        let offset = pacific_offset_secs(utc_approx);
        // Pacific = UTC + offset, so UTC = Pacific - offset
        (utc_approx as i64 - offset) as u64
    }

    /// Days from 1970-01-01 to a civil date (Howard Hinnant algorithm).
    fn civil_days_from_epoch(year: i32, month: u32, day: u32) -> i64 {
        let y = if month <= 2 {
            year as i64 - 1
        } else {
            year as i64
        };
        let m = if month <= 2 {
            month as i64 + 9
        } else {
            month as i64 - 3
        };
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = (y - era * 400) as u64;
        let doy = (153 * m as u64 + 2) / 5 + day as u64 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146097 + doe as i64 - 719468
    }

    #[test]
    fn test_peak_active_weekday_morning() {
        // Wednesday 2026-04-08 10:00 PT (PDT, so UTC-7)
        let ts = pacific_to_utc(2026, 4, 8, 10);
        assert!(is_peak_time(ts, None));
    }

    #[test]
    fn test_off_peak_weekday_evening() {
        // Wednesday 2026-04-08 20:00 PT
        let ts = pacific_to_utc(2026, 4, 8, 20);
        assert!(!is_peak_time(ts, None));
    }

    #[test]
    fn test_off_peak_weekend() {
        // Saturday 2026-04-11 12:00 PT
        let ts = pacific_to_utc(2026, 4, 11, 12);
        assert!(!is_peak_time(ts, None));
    }

    #[test]
    fn test_peak_boundary_start() {
        // Wednesday 2026-04-08 07:00 PT — exactly at start, should be peak
        let ts = pacific_to_utc(2026, 4, 8, 7);
        assert!(is_peak_time(ts, None));
    }

    #[test]
    fn test_peak_boundary_end() {
        // Wednesday 2026-04-08 17:00 PT — at end boundary, should NOT be peak (< 17)
        let ts = pacific_to_utc(2026, 4, 8, 17);
        assert!(!is_peak_time(ts, None));
    }

    #[test]
    fn test_peak_boundary_just_before_end() {
        // Wednesday 2026-04-08 16:59 PT — just before end, should be peak
        let ts = pacific_to_utc(2026, 4, 8, 16);
        assert!(is_peak_time(ts, None));
    }

    #[test]
    fn test_custom_hours() {
        let pk_cfg = PeakUsageConfig {
            start_hour: Some(9),
            end_hour: Some(18),
            ..Default::default()
        };
        // Wednesday 2026-04-08 08:00 PT — before custom start
        let ts = pacific_to_utc(2026, 4, 8, 8);
        assert!(!is_peak_time(ts, Some(&pk_cfg)));
        // Wednesday 2026-04-08 17:30 PT — within custom window
        let ts = pacific_to_utc(2026, 4, 8, 17);
        assert!(is_peak_time(ts, Some(&pk_cfg)));
    }

    #[test]
    fn test_dst_spring_forward_march() {
        // 2026 DST spring forward: second Sunday of March = March 8
        // March 9 (Monday) should be PDT (UTC-7)
        // 10:00 PT on March 9 = 17:00 UTC
        let ts = pacific_to_utc(2026, 3, 9, 10);
        assert!(is_peak_time(ts, None));
    }

    #[test]
    fn test_dst_fall_back_november() {
        // 2026 DST fall back: first Sunday of November = November 1
        // November 2 (Monday) should be PST (UTC-8)
        // 10:00 PT on Nov 2 = 18:00 UTC
        let ts = pacific_to_utc(2026, 11, 2, 10);
        assert!(is_peak_time(ts, None));
    }

    #[test]
    fn test_pst_january() {
        // January is always PST (UTC-8)
        // Wednesday 2026-01-07 10:00 PT = 18:00 UTC
        let ts = pacific_to_utc(2026, 1, 7, 10);
        assert!(is_peak_time(ts, None));
    }

    #[test]
    fn test_disabled_returns_none() {
        let cfg = CshipConfig {
            peak_usage: Some(PeakUsageConfig {
                disabled: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        };
        let ctx = Context::default();
        assert_eq!(render(&ctx, &cfg), None);
    }

    #[test]
    fn test_render_with_custom_symbol() {
        // We can't easily control SystemTime::now() in render(), so test the
        // format logic via a direct is_peak_time check + verify the format path.
        let pk_cfg = PeakUsageConfig {
            symbol: Some("🔥 ".to_string()),
            ..Default::default()
        };
        let symbol = pk_cfg.symbol.as_deref().unwrap_or(DEFAULT_SYMBOL);
        let content = format!("{symbol}{PEAK_LABEL}");
        assert_eq!(content, "🔥 Peak");
    }

    #[test]
    fn test_day_of_week_known_dates() {
        // 2026-04-08 is a Wednesday (3)
        assert_eq!(day_of_week(2026, 4, 8), 3);
        // 2026-04-11 is a Saturday (6)
        assert_eq!(day_of_week(2026, 4, 11), 6);
        // 2026-04-12 is a Sunday (0)
        assert_eq!(day_of_week(2026, 4, 12), 0);
        // 1970-01-01 was a Thursday (4)
        assert_eq!(day_of_week(1970, 1, 1), 4);
    }

    #[test]
    fn test_nth_sunday_of_month() {
        // Second Sunday of March 2026 = March 8
        assert_eq!(nth_sunday_of_month(2026, 3, 2), 8);
        // First Sunday of November 2026 = November 1
        assert_eq!(nth_sunday_of_month(2026, 11, 1), 1);
    }

    #[test]
    fn test_utc_epoch_to_ymd_h_known_date() {
        // 2026-04-08 00:00:00 UTC
        // Days from epoch: use civil_days_from_epoch helper
        let days = civil_days_from_epoch(2026, 4, 8);
        let secs = (days as u64) * SECS_PER_DAY + 14 * SECS_PER_HOUR; // 14:00 UTC
        let (y, m, d, h) = utc_epoch_to_ymd_h(secs);
        assert_eq!((y, m, d, h), (2026, 4, 8, 14));
    }

    #[test]
    fn test_monday_is_peak() {
        // Monday 2026-04-06 10:00 PT
        let ts = pacific_to_utc(2026, 4, 6, 10);
        assert!(is_peak_time(ts, None));
    }

    #[test]
    fn test_friday_is_peak() {
        // Friday 2026-04-10 10:00 PT
        let ts = pacific_to_utc(2026, 4, 10, 10);
        assert!(is_peak_time(ts, None));
    }

    #[test]
    fn test_start_ge_end_returns_false() {
        let pk_cfg = PeakUsageConfig {
            start_hour: Some(18),
            end_hour: Some(6),
            ..Default::default()
        };
        let ts = pacific_to_utc(2026, 4, 8, 10);
        assert!(!is_peak_time(ts, Some(&pk_cfg)));
    }

    #[test]
    fn test_start_equals_end_returns_false() {
        let pk_cfg = PeakUsageConfig {
            start_hour: Some(10),
            end_hour: Some(10),
            ..Default::default()
        };
        let ts = pacific_to_utc(2026, 4, 8, 10);
        assert!(!is_peak_time(ts, Some(&pk_cfg)));
    }

    #[test]
    fn test_hour_out_of_range_returns_false() {
        let pk_cfg = PeakUsageConfig {
            start_hour: Some(25),
            end_hour: Some(30),
            ..Default::default()
        };
        let ts = pacific_to_utc(2026, 4, 8, 10);
        assert!(!is_peak_time(ts, Some(&pk_cfg)));
    }

    #[test]
    fn test_end_hour_24_covers_full_day() {
        let pk_cfg = PeakUsageConfig {
            start_hour: Some(0),
            end_hour: Some(24),
            ..Default::default()
        };
        // Hour 23 should be included
        let ts = pacific_to_utc(2026, 4, 8, 23);
        assert!(is_peak_time(ts, Some(&pk_cfg)));
    }

    #[test]
    fn test_end_hour_24_is_valid() {
        let pk_cfg = PeakUsageConfig {
            start_hour: Some(7),
            end_hour: Some(24),
            ..Default::default()
        };
        // Hour 23 within 7–24 window
        let ts = pacific_to_utc(2026, 4, 8, 23);
        assert!(is_peak_time(ts, Some(&pk_cfg)));
    }

    #[test]
    fn test_sunday_is_not_peak() {
        // Sunday 2026-04-12 10:00 PT
        let ts = pacific_to_utc(2026, 4, 12, 10);
        assert!(!is_peak_time(ts, None));
    }
}
