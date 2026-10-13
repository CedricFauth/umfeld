//! Day boundaries in a place's time zone.

use std::str::FromStr;

use chrono::{DateTime, NaiveDate, TimeDelta, TimeZone};
use chrono_tz::Tz;

/// Falls back to UTC if chrono-tz doesn't know the name.
pub fn parse_tz(name: &str) -> Tz {
    Tz::from_str(name).unwrap_or_else(|err| {
        tracing::warn!("error getting timezone, using UTC: {err}");
        chrono_tz::UTC
    })
}

/// First instant of `date` in `tz`. Usually 00:00; later if a DST gap swallows midnight.
pub fn local_midnight(tz: &Tz, date: NaiveDate) -> DateTime<Tz> {
    let midnight = date.and_hms_opt(0, 0, 0).unwrap();
    (0..=24)
        .find_map(|quarter| {
            tz.from_local_datetime(&(midnight + TimeDelta::minutes(15 * quarter)))
                .earliest()
        })
        .unwrap_or_else(|| tz.from_utc_datetime(&midnight))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn midnight(tz: Tz, y: i32, m: u32, d: u32) -> i64 {
        local_midnight(&tz, NaiveDate::from_ymd_opt(y, m, d).unwrap()).timestamp()
    }

    #[test]
    fn dst_days_are_23_and_25_hours() {
        let berlin = chrono_tz::Europe::Berlin;
        assert_eq!(midnight(berlin, 2026, 10, 3), 1790978400);
        assert_eq!(
            midnight(berlin, 2026, 10, 26) - midnight(berlin, 2026, 10, 25),
            25 * 3600
        );
        assert_eq!(
            midnight(berlin, 2026, 3, 30) - midnight(berlin, 2026, 3, 29),
            23 * 3600
        );
    }

    #[test]
    fn midnight_inside_dst_gap() {
        // Chile springs forward at 24:00 -> 01:00
        let santiago = chrono_tz::America::Santiago;
        let start = local_midnight(&santiago, NaiveDate::from_ymd_opt(2026, 9, 6).unwrap());
        assert_eq!(start.format("%H:%M").to_string(), "01:00");
    }
}
