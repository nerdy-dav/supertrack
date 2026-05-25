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
