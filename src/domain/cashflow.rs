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
