use chrono::NaiveDate;
use crate::domain::deadline::business_days_remaining;

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum ComplianceStatus {
    Compliant,   // paid and confirmed within window
    AtRisk,      // <= 2 business days remaining, not paid
    Overdue,     // deadline passed, not paid
    PaidLate,    // paid after deadline
    Pending,     // not yet due, not yet paid
}

impl ComplianceStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Compliant => "Compliant",
            Self::AtRisk => "At Risk",
            Self::Overdue => "OVERDUE",
            Self::PaidLate => "Paid Late",
            Self::Pending => "Pending",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Compliant => "🟢",
            Self::AtRisk => "🟡",
            Self::Overdue => "🔴",
            Self::PaidLate => "⚫",
            Self::Pending => "⚪",
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Compliant => "status-compliant",
            Self::AtRisk => "status-at-risk",
            Self::Overdue => "status-overdue",
            Self::PaidLate => "status-paid-late",
            Self::Pending => "status-pending",
        }
    }
}

impl std::fmt::Display for ComplianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

pub fn determine_status(
    super_deadline: NaiveDate,
    payment_date: Option<NaiveDate>,
    state: &str,
) -> ComplianceStatus {
    match payment_date {
        Some(paid) => {
            if paid <= super_deadline {
                ComplianceStatus::Compliant
            } else {
                ComplianceStatus::PaidLate
            }
        }
        None => {
            let bdays = business_days_remaining(super_deadline, state);
            if bdays < 0 {
                ComplianceStatus::Overdue
            } else if bdays <= 2 {
                ComplianceStatus::AtRisk
            } else {
                ComplianceStatus::Pending
            }
        }
    }
}

/// Estimate SGC penalty for late super payment
/// SGC = unpaid super + interest (10% p.a.) + $20 admin fee per employee per quarter
pub fn estimate_sgc_penalty(amount_owing: f64, days_late: u32, employee_count: u32) -> f64 {
    let interest = amount_owing * 0.10 * (days_late as f64 / 365.0);
    let admin = 20.0 * employee_count as f64;
    amount_owing + interest + admin
}
