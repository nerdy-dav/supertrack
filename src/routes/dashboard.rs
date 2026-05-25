use axum::extract::State;
use axum::response::Html;
use std::sync::Arc;
use chrono::NaiveDate;
use minijinja::context;

use crate::AppState;
use crate::domain::compliance::determine_status;
use crate::domain::deadline::business_days_remaining;

pub async fn index(State(state): State<Arc<AppState>>) -> Html<String> {
    let business = state.db.get_business();
    let state_code = business.as_ref().map(|b| b.state.clone()).unwrap_or("VIC".to_string());
    let setup_complete = business.is_some();

    let runs_with_payments = state.db.list_all_payments();
    let today = chrono::Local::now().date_naive();

    let rows: Vec<serde_json::Value> = runs_with_payments.iter().map(|(run, payment)| {
        let deadline = NaiveDate::parse_from_str(&run.super_deadline, "%Y-%m-%d").unwrap_or(today);
        let paid_date = payment.as_ref().and_then(|p| NaiveDate::parse_from_str(&p.payment_date, "%Y-%m-%d").ok());
        let status = determine_status(deadline, paid_date, &state_code);
        let bdays = business_days_remaining(deadline, &state_code);

        serde_json::json!({
            "id": run.id,
            "pay_date": run.pay_date,
            "period_start": run.period_start,
            "period_end": run.period_end,
            "total_super_owing": format!("{:.2}", run.total_super_owing),
            "super_deadline": run.super_deadline,
            "status_label": status.label(),
            "status_emoji": status.emoji(),
            "status_css": status.css_class(),
            "business_days_remaining": bdays,
            "has_payment": payment.is_some(),
            "payment_date": payment.as_ref().map(|p| p.payment_date.clone()).unwrap_or_default(),
        })
    }).collect();

    let overdue_count = rows.iter().filter(|r| r["status_label"] == "OVERDUE").count();
    let at_risk_count = rows.iter().filter(|r| r["status_label"] == "At Risk").count();

    let payday_super_start = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
    let days_to_payday_super = (payday_super_start - today).num_days();
    let payday_super_active = today >= payday_super_start;

    let ctx = context! {
        business => business,
        setup_complete => setup_complete,
        pay_runs => rows,
        overdue_count => overdue_count,
        at_risk_count => at_risk_count,
        today => today.to_string(),
        days_to_payday_super => days_to_payday_super,
        payday_super_active => payday_super_active,
    };

    crate::routes::render(&state.env, "dashboard.html", ctx)
}
