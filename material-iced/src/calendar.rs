// SPDX-License-Identifier: LGPL-3.0-only

//! Gregorian calendar dates and the names used by the pickers.

/// A date in the proleptic Gregorian calendar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    /// Year, such as 2025.
    pub year: i32,
    /// Month from 1 to 12.
    pub month: u8,
    /// Day of the month, from 1.
    pub day: u8,
}

/// Whether `year` is a leap year.
pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Number of days in `month` of `year`.
pub fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

impl Date {
    /// A date, or `None` when the month or day does not exist.
    pub fn new(year: i32, month: u8, day: u8) -> Option<Date> {
        ((1..=12).contains(&month) && day >= 1 && day <= days_in_month(year, month))
            .then_some(Date { year, month, day })
    }

    /// Days since 1970-01-01.
    pub fn to_days(self) -> i64 {
        let y = i64::from(self.year) - i64::from(self.month <= 2);
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let m = i64::from(self.month);
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(self.day) - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    /// The date `days` days after 1970-01-01.
    pub fn from_days(days: i64) -> Date {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
        let year = (yoe + era * 400 + i64::from(month <= 2)) as i32;
        Date { year, month, day }
    }

    /// Day of the week, 0 for Sunday to 6 for Saturday.
    pub fn weekday(self) -> u8 {
        (self.to_days() + 4).rem_euclid(7) as u8
    }

    /// The date `days` days later, or earlier when negative.
    pub fn add_days(self, days: i64) -> Date {
        Date::from_days(self.to_days() + days)
    }

    /// The date `months` months later, with the day kept or clamped to the month length.
    pub fn add_months(self, months: i32) -> Date {
        let index = self.year * 12 + i32::from(self.month) - 1 + months;
        let year = index.div_euclid(12);
        let month = index.rem_euclid(12) as u8 + 1;
        Date {
            year,
            month,
            day: self.day.min(days_in_month(year, month)),
        }
    }
}

/// Names shown by the date picker. The defaults are English.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Names {
    /// Full month names from January.
    pub months: [&'static str; 12],
    /// Short month names from January.
    pub months_short: [&'static str; 12],
    /// Short weekday names from Sunday.
    pub weekdays_short: [&'static str; 7],
    /// One letter weekday names from Sunday.
    pub weekday_initials: [&'static str; 7],
    /// First day of the week, 0 for Sunday to 6 for Saturday.
    pub first_weekday: u8,
}

impl Default for Names {
    fn default() -> Self {
        Names {
            months: [
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ],
            months_short: [
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
            ],
            weekdays_short: ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
            weekday_initials: ["S", "M", "T", "W", "T", "F", "S"],
            first_weekday: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_and_known_weekdays() {
        let epoch = Date::new(1970, 1, 1).unwrap();
        assert_eq!(epoch.to_days(), 0);
        assert_eq!(epoch.weekday(), 4);
        assert_eq!(Date::new(2000, 1, 1).unwrap().weekday(), 6);
        assert_eq!(Date::new(2024, 2, 29).unwrap().weekday(), 4);
        assert_eq!(Date::new(1900, 3, 1).unwrap().weekday(), 4);
    }

    #[test]
    fn days_round_trip_over_four_centuries() {
        let start = Date::new(1900, 1, 1).unwrap().to_days();
        let mut previous = Date::from_days(start);
        for days in start + 1..start + 146_097 {
            let date = Date::from_days(days);
            assert_eq!(date.to_days(), days);
            assert!(date > previous);
            assert!(Date::new(date.year, date.month, date.day).is_some());
            previous = date;
        }
    }

    #[test]
    fn leap_years_and_month_lengths() {
        assert!(is_leap_year(2000) && is_leap_year(2024));
        assert!(!is_leap_year(1900) && !is_leap_year(2023));
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2023, 4), 30);
        assert!(Date::new(2023, 2, 29).is_none());
        assert!(Date::new(2023, 13, 1).is_none());
        assert!(Date::new(2023, 1, 0).is_none());
    }

    #[test]
    fn month_arithmetic_clamps_and_wraps_years() {
        let date = Date::new(2024, 1, 31).unwrap();
        assert_eq!(date.add_months(1), Date::new(2024, 2, 29).unwrap());
        assert_eq!(date.add_months(-1), Date::new(2023, 12, 31).unwrap());
        assert_eq!(date.add_months(12), Date::new(2025, 1, 31).unwrap());
        assert_eq!(date.add_months(-13), Date::new(2022, 12, 31).unwrap());
        assert_eq!(date.add_days(1), Date::new(2024, 2, 1).unwrap());
        assert_eq!(date.add_days(-31), Date::new(2023, 12, 31).unwrap());
    }
}
