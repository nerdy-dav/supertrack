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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliant_status() {
        let deadline = NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();
        let paid = Some(NaiveDate::from_ymd_opt(2026, 6, 25).unwrap());
        assert_eq!(determine_status(deadline, paid, "VIC"), ComplianceStatus::Compliant);
    }

    #[test]
    fn test_compliant_on_deadline() {
        let deadline = NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();
        let paid = Some(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap());
        assert_eq!(determine_status(deadline, paid, "VIC"), ComplianceStatus::Compliant);
    }

    #[test]
    fn test_paid_late() {
        let deadline = NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();
        let paid = Some(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap());
        assert_eq!(determine_status(deadline, paid, "VIC"), ComplianceStatus::PaidLate);
    }

    #[test]
    fn test_compliance_status_labels() {
        assert_eq!(ComplianceStatus::Compliant.label(), "Compliant");
        assert_eq!(ComplianceStatus::AtRisk.label(), "At Risk");
        assert_eq!(ComplianceStatus::Overdue.label(), "OVERDUE");
        assert_eq!(ComplianceStatus::PaidLate.label(), "Paid Late");
        assert_eq!(ComplianceStatus::Pending.label(), "Pending");
    }

    #[test]
    fn test_compliance_status_emoji() {
        assert_eq!(ComplianceStatus::Compliant.emoji(), "🟢");
        assert_eq!(ComplianceStatus::AtRisk.emoji(), "🟡");
        assert_eq!(ComplianceStatus::Overdue.emoji(), "🔴");
        assert_eq!(ComplianceStatus::PaidLate.emoji(), "⚫");
        assert_eq!(ComplianceStatus::Pending.emoji(), "⚪");
    }

    #[test]
    fn test_compliance_status_css() {
        assert_eq!(ComplianceStatus::Compliant.css_class(), "status-compliant");
        assert_eq!(ComplianceStatus::AtRisk.css_class(), "status-at-risk");
        assert_eq!(ComplianceStatus::Overdue.css_class(), "status-overdue");
        assert_eq!(ComplianceStatus::PaidLate.css_class(), "status-paid-late");
        assert_eq!(ComplianceStatus::Pending.css_class(), "status-pending");
    }

    #[test]
    fn test_compliance_status_display() {
        assert_eq!(format!("{}", ComplianceStatus::Compliant), "Compliant");
        assert_eq!(format!("{}", ComplianceStatus::Overdue), "OVERDUE");
    }

    #[test]
    fn test_compliance_status_debug_clone_partial_eq() {
        let s = ComplianceStatus::Compliant;
        assert_eq!(s.clone(), s);
        assert!(format!("{:?}", s).contains("Compliant"));
    }
}

