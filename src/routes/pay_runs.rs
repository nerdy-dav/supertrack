use axum::extract::{State, Path};
use axum::response::{Html, Redirect};
use axum::Form;
use std::sync::Arc;
use minijinja::context;
use serde::Deserialize;
use chrono::NaiveDate;

use crate::AppState;
use crate::db::PayRun;
use crate::domain::deadline::calculate_deadline;
use crate::domain::compliance::determine_status;

#[derive(Deserialize)]
pub struct PayRunForm {
    pub pay_date: String,
    pub period_start: String,
    pub period_end: String,
    pub total_wages: f64,
    pub stp_reference: Option<String>,
}

pub async fn list(State(state): State<Arc<AppState>>) -> Html<String> {
    let business = state.db.get_business();
    let state_code = business.as_ref().map(|b| b.state.clone()).unwrap_or("VIC".to_string());
    let sgc_rate = business.as_ref().map(|b| b.sgc_rate).unwrap_or(0.12);
    let today = chrono::Local::now().date_naive();

    let runs = state.db.list_pay_runs();
    let rows: Vec<serde_json::Value> = runs.iter().map(|run| {
        let deadline = NaiveDate::parse_from_str(&run.super_deadline, "%Y-%m-%d").unwrap_or(today);
        let payment = state.db.get_payment_for_run(run.id);
        let paid_date = payment.as_ref().and_then(|p| NaiveDate::parse_from_str(&p.payment_date, "%Y-%m-%d").ok());
        let status = determine_status(deadline, paid_date, &state_code);
        serde_json::json!({
            "id": run.id,
            "pay_date": run.pay_date,
            "period_start": run.period_start,
            "period_end": run.period_end,
            "total_wages": format!("{:.2}", run.total_wages),
            "total_super_owing": format!("{:.2}", run.total_super_owing),
            "super_deadline": run.super_deadline,
            "status_label": status.label(),
            "status_emoji": status.emoji(),
            "status_css": status.css_class(),
            "has_payment": payment.is_some(),
        })
    }).collect();

    let ctx = context! { pay_runs => rows, sgc_rate_pct => sgc_rate * 100.0 };
    crate::routes::render(&state.env, "pay_runs.html", ctx)
}

pub async fn new_form(State(state): State<Arc<AppState>>) -> Html<String> {
    let today = chrono::Local::now().date_naive();
    let employees = state.db.list_employees();
    let business = state.db.get_business();
    let sgc_rate = business.as_ref().map(|b| b.sgc_rate).unwrap_or(0.12);
    let ctx = context! { today => today.to_string(), employees => employees, sgc_rate_pct => sgc_rate * 100.0 };
    crate::routes::render(&state.env, "pay_run_form.html", ctx)
}

pub async fn create(State(state): State<Arc<AppState>>, Form(form): Form<PayRunForm>) -> Redirect {
    let business = state.db.get_business();
    let state_code = business.as_ref().map(|b| b.state.clone()).unwrap_or("VIC".to_string());
    let sgc_rate = business.as_ref().map(|b| b.sgc_rate).unwrap_or(0.12);
    let lag = business.as_ref().map(|b| b.clearing_house_lag_days).unwrap_or(1) as u32;

    let pay_date = NaiveDate::parse_from_str(&form.pay_date, "%Y-%m-%d")
        .unwrap_or(chrono::Local::now().date_naive());
    let deadline = calculate_deadline(pay_date, &state_code, lag);
    let total_super = form.total_wages * sgc_rate;

    let run = PayRun {
        id: 0,
        pay_date: form.pay_date,
        period_start: form.period_start,
        period_end: form.period_end,
        total_wages: form.total_wages,
        total_super_owing: total_super,
        super_deadline: deadline.to_string(),
        stp_reference: form.stp_reference,
        created_at: String::new(),
    };
    state.db.create_pay_run(&run).ok();
    Redirect::to("/pay-runs")
}

pub async fn show(State(state): State<Arc<AppState>>, Path(id): Path<i64>) -> Html<String> {
    let business = state.db.get_business();
    let state_code = business.as_ref().map(|b| b.state.clone()).unwrap_or("VIC".to_string());
    let today = chrono::Local::now().date_naive();

    if let Some(run) = state.db.get_pay_run(id) {
        let deadline = NaiveDate::parse_from_str(&run.super_deadline, "%Y-%m-%d").unwrap_or(today);
        let payment = state.db.get_payment_for_run(id);
        let paid_date = payment.as_ref().and_then(|p| NaiveDate::parse_from_str(&p.payment_date, "%Y-%m-%d").ok());
        let status = determine_status(deadline, paid_date, &state_code);
        let days_remaining = (deadline - today).num_days();
        let run_json = serde_json::json!({
            "id": run.id,
            "pay_date": run.pay_date,
            "period_start": run.period_start,
            "period_end": run.period_end,
            "total_wages": format!("{:.2}", run.total_wages),
            "total_super_owing": format!("{:.2}", run.total_super_owing),
            "super_deadline": run.super_deadline,
            "stp_reference": run.stp_reference,
            "created_at": run.created_at,
        });
        let payment_json = payment.as_ref().map(|p| serde_json::json!({
            "payment_date": p.payment_date,
            "amount_paid": format!("{:.2}", p.amount_paid),
            "clearing_house_receipt": p.clearing_house_receipt,
            "fund_receipt_confirmed": if p.fund_receipt_confirmed { "Yes" } else { "Pending" },
        }));
        let ctx = context! {
            run => run_json,
            payment => payment_json,
            status_label => status.label(),
            status_emoji => status.emoji(),
            status_css => status.css_class(),
            days_remaining => days_remaining,
        };
        return crate::routes::render(&state.env, "pay_run_detail.html", ctx);
    }
    crate::routes::render(&state.env, "pay_run_detail.html", context! {})
}
