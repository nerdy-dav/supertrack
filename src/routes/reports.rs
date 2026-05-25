use axum::extract::State;
use axum::response::{Html, Response};
use axum::body::Body;
use std::sync::Arc;
use minijinja::context;
use chrono::NaiveDate;

use crate::AppState;
use crate::domain::compliance::determine_status;

pub async fn index(State(state): State<Arc<AppState>>) -> Html<String> {
    crate::routes::render(&state.env, "reports.html", context! {})
}

pub async fn export_csv(State(state): State<Arc<AppState>>) -> Response<Body> {
    let business = state.db.get_business();
    let state_code = business.as_ref().map(|b| b.state.clone()).unwrap_or("VIC".to_string());
    let biz_name = business.as_ref().map(|b| b.name.clone()).unwrap_or("Unknown".to_string());
    let abn = business.as_ref().map(|b| b.abn.clone()).unwrap_or_default();
    let today = chrono::Local::now().date_naive();

    let mut csv = format!("SuperTrack AU Compliance Export\nBusiness: {}\nABN: {}\nGenerated: {}\n\n", biz_name, abn, today);
    csv.push_str("Pay Date,Period Start,Period End,Total Wages,Super Owing,Deadline,Payment Date,Amount Paid,Receipt Ref,Status\n");

    for (run, payment) in state.db.list_all_payments() {
        let deadline = NaiveDate::parse_from_str(&run.super_deadline, "%Y-%m-%d").unwrap_or(today);
        let paid_date = payment.as_ref().and_then(|p| NaiveDate::parse_from_str(&p.payment_date, "%Y-%m-%d").ok());
        let status = determine_status(deadline, paid_date, &state_code);
        csv.push_str(&format!(
            "{},{},{},{:.2},{:.2},{},{},{:.2},{},{}\n",
            run.pay_date, run.period_start, run.period_end,
            run.total_wages, run.total_super_owing, run.super_deadline,
            payment.as_ref().map(|p| p.payment_date.as_str()).unwrap_or(""),
            payment.as_ref().map(|p| p.amount_paid).unwrap_or(0.0),
            payment.as_ref().and_then(|p| p.clearing_house_receipt.as_deref()).unwrap_or(""),
            status.label()
        ));
    }

    Response::builder()
        .header("Content-Type", "text/csv")
        .header("Content-Disposition", "attachment; filename=\"supertrack_export.csv\"")
        .body(Body::from(csv))
        .unwrap()
}
