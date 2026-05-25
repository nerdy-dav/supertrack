use chrono::{NaiveDate, Datelike, Weekday, Duration};

/// Australian public holidays (national + per state) 2024-2027
/// Format: (year, month, day, states) where states is "" for national
fn public_holidays() -> Vec<(i32, u32, u32, &'static str)> {
    vec![
        // National
        (2026, 1, 1, ""),   // New Year's Day
        (2026, 1, 26, ""),  // Australia Day
        (2026, 4, 3, ""),   // Good Friday
        (2026, 4, 4, ""),   // Easter Saturday
        (2026, 4, 5, ""),   // Easter Sunday
        (2026, 4, 6, ""),   // Easter Monday
        (2026, 4, 25, ""),  // Anzac Day
        (2026, 12, 25, ""), // Christmas Day
        (2026, 12, 26, ""), // Boxing Day
        (2026, 12, 28, ""), // Boxing Day substitute
        // Queen's Birthday varies by state (June for most, October for QLD)
        (2026, 6, 8, "NSW,VIC,SA,TAS,ACT,NT"),
        (2026, 10, 5, "QLD"),
        (2026, 9, 28, "WA"),
        // State specific
        (2026, 11, 3, "VIC"),  // Melbourne Cup Day
        (2026, 8, 12, "QLD"),  // Royal Queensland Show (Brisbane)
        (2026, 3, 9, "ACT"),   // Canberra Day
        (2026, 5, 25, "ACT"),  // Reconciliation Day
        (2026, 3, 2, "SA"),    // Adelaide Cup
        (2026, 6, 1, "WA"),    // WA Day
        // 2027
        (2027, 1, 1, ""),
        (2027, 1, 26, ""),
        (2027, 3, 26, ""),  // Good Friday 2027
        (2027, 3, 29, ""),  // Easter Monday 2027
        (2027, 4, 25, ""),
        (2027, 6, 14, "NSW,VIC,SA,TAS,ACT,NT"), // Queen's Birthday 2027
        (2027, 12, 25, ""),
        (2027, 12, 26, ""),
        (2027, 12, 27, ""),
        (2027, 11, 2, "VIC"), // Melbourne Cup 2027
    ]
}

pub fn is_public_holiday(date: NaiveDate, state: &str) -> bool {
    let holidays = public_holidays();
    for (y, m, d, states) in holidays {
        if date.year() == y && date.month() == m && date.day() == d {
            if states.is_empty() {
                return true; // national
            }
            if states.contains(state) {
                return true;
            }
        }
    }
    false
}

pub fn is_business_day(date: NaiveDate, state: &str) -> bool {
    let wd = date.weekday();
    if wd == Weekday::Sat || wd == Weekday::Sun {
        return false;
    }
    !is_public_holiday(date, state)
}

/// Calculate the super deadline: pay_date + 7 business days, minus clearing_lag business days
pub fn calculate_deadline(pay_date: NaiveDate, state: &str, clearing_lag_days: u32) -> NaiveDate {
    // Step 1: Add 7 business days
    let mut business_days = 0u32;
    let mut current = pay_date;
    while business_days < 7 {
        current = current + Duration::days(1);
        if is_business_day(current, state) {
            business_days += 1;
        }
    }

    // Step 2: Subtract clearing lag so payment leaves bank account in time
    let mut lag_remaining = clearing_lag_days;
    while lag_remaining > 0 {
        current = current - Duration::days(1);
        if is_business_day(current, state) {
            lag_remaining -= 1;
        }
    }

    current
}

/// Business days remaining
pub fn business_days_remaining(deadline: NaiveDate, state: &str) -> i64 {
    let today = chrono::Local::now().date_naive();
    if deadline <= today {
        return (deadline - today).num_days(); // negative
    }
    let mut count = 0i64;
    let mut current = today;
    while current < deadline {
        current = current + Duration::days(1);
        if is_business_day(current, state) {
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_national_holiday() {
        let d = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        assert!(is_public_holiday(d, "NSW"));
        assert!(is_public_holiday(d, "VIC"));
        assert!(is_public_holiday(d, "QLD"));
    }

    #[test]
    fn test_state_specific_holiday() {
        let melbourne_cup = NaiveDate::from_ymd_opt(2026, 11, 3).unwrap();
        assert!(is_public_holiday(melbourne_cup, "VIC"));
        assert!(!is_public_holiday(melbourne_cup, "NSW"));
        assert!(!is_public_holiday(melbourne_cup, "QLD"));
    }

    #[test]
    fn test_not_a_holiday() {
        let d = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        assert!(!is_public_holiday(d, "VIC"));
        assert!(!is_public_holiday(d, "NSW"));
    }

    #[test]
    fn test_weekend_is_not_business_day() {
        let sat = NaiveDate::from_ymd_opt(2026, 5, 30).unwrap(); // Saturday
        let sun = NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(); // Sunday
        assert!(!is_business_day(sat, "VIC"));
        assert!(!is_business_day(sun, "VIC"));
    }

    #[test]
    fn test_holiday_is_not_business_day() {
        let xmas = NaiveDate::from_ymd_opt(2026, 12, 25).unwrap();
        assert!(!is_business_day(xmas, "VIC"));
    }

    #[test]
    fn test_weekday_is_business_day() {
        let wed = NaiveDate::from_ymd_opt(2026, 6, 17).unwrap();
        assert!(is_business_day(wed, "VIC"));
    }

    #[test]
    fn test_calculate_deadline_basic() {
        let pay_date = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(); // Monday
        let deadline = calculate_deadline(pay_date, "VIC", 0);
        // 7 business days from Mon Jun 1:
        // Jun 2 Tue (+1), 3 Wed (+2), 4 Thu (+3), 5 Fri (+4)
        // Jun 6-7 weekend, Jun 8 Queen's Birthday VIC holiday
        // Jun 9 Tue (+5), 10 Wed (+6), 11 Thu (+7)
        assert_eq!(deadline, NaiveDate::from_ymd_opt(2026, 6, 11).unwrap());
    }

    #[test]
    fn test_calculate_deadline_with_lag() {
        let pay_date = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(); // Monday
        let deadline = calculate_deadline(pay_date, "VIC", 1);
        // 7 business days = Jun 11, minus 1 biz day lag = Jun 10
        assert_eq!(deadline, NaiveDate::from_ymd_opt(2026, 6, 10).unwrap());
    }

    #[test]
    fn test_calculate_deadline_crosses_holiday() {
        let pay_date = NaiveDate::from_ymd_opt(2026, 12, 21).unwrap(); // Monday
        let deadline = calculate_deadline(pay_date, "VIC", 0);
        // 7 business days from Dec 21:
        // Dec 22 Tue (+1), 23 Wed (+2), 24 Thu (+3)
        // Dec 25 Fri XMAS holiday, Dec 26 Sat, 27 Sun
        // Dec 28 Mon Boxing Day sub holiday, Dec 29 Tue (+4), 30 Wed (+5), 31 Thu (+6)
        // Jan 1 Fri New Year holiday, Jan 2 Sat, 3 Sun
        // Jan 4 Mon (+7)
        assert_eq!(deadline, NaiveDate::from_ymd_opt(2027, 1, 4).unwrap());
    }

    #[test]
    fn test_calculate_deadline_friday_pay() {
        let pay_date = NaiveDate::from_ymd_opt(2026, 6, 5).unwrap(); // Friday
        let deadline = calculate_deadline(pay_date, "VIC", 0);
        // Fri Jun 5: +7 business days
        // Jun 6 Sat skip, Jun 7 Sun skip
        // Jun 8 Mon (+1), Jun 8 - Queen's Birthday in VIC? Actually in 2026 Queen's Birthday is Jun 8.
        // Wait, looking at the holiday data: (2026, 6, 8, "NSW,VIC,SA,TAS,ACT,NT") - yes, it's a holiday in VIC
        // So: Jun 8 is a holiday in VIC, skip
        // Jun 9 Tue (+1), Jun 10 Wed (+2), Jun 11 Thu (+3), Jun 12 Fri (+4)
        // Jun 13 Sat skip, Jun 14 Sun skip
        // Jun 15 Mon (+5), Jun 16 Tue (+6), Jun 17 Wed (+7)
        assert_eq!(deadline, NaiveDate::from_ymd_opt(2026, 6, 17).unwrap());
    }

    #[test]
    fn test_queens_birthday_holiday_vic() {
        let qb = NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        assert!(is_public_holiday(qb, "VIC"));
        assert!(is_public_holiday(qb, "NSW"));
        assert!(!is_public_holiday(qb, "QLD"));
        assert!(!is_public_holiday(qb, "WA"));
    }

    #[test]
    fn test_2027_holidays() {
        let nyd = NaiveDate::from_ymd_opt(2027, 1, 1).unwrap();
        assert!(is_public_holiday(nyd, "VIC"));

        let melb_cup_2027 = NaiveDate::from_ymd_opt(2027, 11, 2).unwrap();
        assert!(is_public_holiday(melb_cup_2027, "VIC"));
        assert!(!is_public_holiday(melb_cup_2027, "NSW"));
    }
}
