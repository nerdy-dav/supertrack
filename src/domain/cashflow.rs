#[derive(Debug, serde::Serialize)]
pub struct CashFlowImpact {
    pub monthly_wage_bill: f64,
    pub quarterly_super_lump: f64,
    pub per_fortnight_super: f64,
    pub per_week_super: f64,
    pub working_capital_shift: f64,
    pub recommended_buffer: f64,
    pub annual_super_total: f64,
}

pub fn calculate_impact(monthly_wage_bill: f64, pay_frequency: &str, sgc_rate: f64) -> CashFlowImpact {
    let annual_wages = monthly_wage_bill * 12.0;
    let annual_super = annual_wages * sgc_rate;
    let quarterly_lump = annual_super / 4.0;
    let per_fortnight = annual_super / 26.0;
    let per_week = annual_super / 52.0;

    let working_capital_shift = match pay_frequency {
        "weekly" => quarterly_lump - per_week,
        "fortnightly" => quarterly_lump - per_fortnight,
        "monthly" => quarterly_lump - (annual_super / 12.0),
        _ => quarterly_lump - per_fortnight,
    };

    // Recommended buffer: 1.5x the per-run super amount
    let per_run = match pay_frequency {
        "weekly" => per_week,
        "fortnightly" => per_fortnight,
        "monthly" => annual_super / 12.0,
        _ => per_fortnight,
    };

    CashFlowImpact {
        monthly_wage_bill,
        quarterly_super_lump: quarterly_lump,
        per_fortnight_super: per_fortnight,
        per_week_super: per_week,
        working_capital_shift: working_capital_shift.abs(),
        recommended_buffer: per_run * 1.5,
        annual_super_total: annual_super,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.001
    }

    #[test]
    fn test_impact_fortnightly() {
        let result = calculate_impact(45000.0, "fortnightly", 0.12);
        assert!(approx_eq(result.monthly_wage_bill, 45000.0));
        assert!(approx_eq(result.annual_super_total, 64800.0));
        assert!(approx_eq(result.quarterly_super_lump, 16200.0));
        assert!(approx_eq(result.per_fortnight_super, 2492.3077));
        assert!(approx_eq(result.per_week_super, 1246.1538));
        assert!(approx_eq(result.working_capital_shift, 13707.6923));
        assert!(approx_eq(result.recommended_buffer, 3738.4615));
    }

    #[test]
    fn test_impact_weekly() {
        let result = calculate_impact(45000.0, "weekly", 0.12);
        assert!(approx_eq(result.annual_super_total, 64800.0));
        assert!(approx_eq(result.quarterly_super_lump, 16200.0));
        assert!(approx_eq(result.per_week_super, 1246.1538));
        assert!(approx_eq(result.working_capital_shift, 14953.8462));
        assert!(approx_eq(result.recommended_buffer, 1869.2307));
    }

    #[test]
    fn test_impact_monthly() {
        let result = calculate_impact(45000.0, "monthly", 0.12);
        assert!(approx_eq(result.annual_super_total, 64800.0));
        assert!(approx_eq(result.per_fortnight_super, 2492.3077));
        assert!(approx_eq(result.working_capital_shift, 10800.0));
        assert!(approx_eq(result.recommended_buffer, 8100.0));
    }

    #[test]
    fn test_impact_unknown_frequency_defaults_to_fortnightly() {
        let result = calculate_impact(10000.0, "unknown", 0.12);
        let fortnightly = calculate_impact(10000.0, "fortnightly", 0.12);
        assert_eq!(result.per_fortnight_super, fortnightly.per_fortnight_super);
        assert_eq!(result.working_capital_shift, fortnightly.working_capital_shift);
        assert_eq!(result.recommended_buffer, fortnightly.recommended_buffer);
    }

    #[test]
    fn test_impact_zero_wages() {
        let result = calculate_impact(0.0, "fortnightly", 0.12);
        assert!(approx_eq(result.annual_super_total, 0.0));
        assert!(approx_eq(result.working_capital_shift, 0.0));
        assert!(approx_eq(result.recommended_buffer, 0.0));
    }

    #[test]
    fn test_impact_different_sgc_rate() {
        let result = calculate_impact(10000.0, "fortnightly", 0.115);
        assert!(approx_eq(result.annual_super_total, 13800.0));
        assert!(approx_eq(result.quarterly_super_lump, 3450.0));
    }

    #[test]
    fn test_impact_serde_serialize() {
        let result = calculate_impact(50000.0, "fortnightly", 0.12);
        let json = serde_json::to_value(&result).unwrap();
        assert!(json.get("annual_super_total").is_some());
        assert!(json.get("working_capital_shift").is_some());
    }
}
