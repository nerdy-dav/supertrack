use axum::extract::{State, Path};
use axum::response::{Html, Redirect};
use axum::Form;
use std::sync::Arc;
use minijinja::context;
use serde::Deserialize;
use chrono::NaiveDate;

use crate::AppState;
use crate::db::SuperPayment;
use crate::domain::compliance::determine_status;

#[derive(Deserialize)]
pub struct PaymentForm {
    pub payment_date: String,
    pub clearing_house_receipt: Option<String>,
    pub amount_paid: f64,
}

pub async fn new_form(State(state): State<Arc<AppState>>, Path(pay_run_id): Path<i64>) -> Html<String> {
    let today = chrono::Local::now().date_naive();
    let (super_owing, super_deadline) = state.db.get_pay_run(pay_run_id)
        .map(|r| (r.total_super_owing, r.super_deadline))
        .unwrap_or((0.0, String::new()));
    let ctx = context! { today => today.to_string(), pay_run_id => pay_run_id, super_owing => super_owing, super_deadline => super_deadline };
    crate::routes::render(&state.env, "payment_form.html", ctx)
}

pub async fn create(State(state): State<Arc<AppState>>, Path(pay_run_id): Path<i64>, Form(form): Form<PaymentForm>) -> Redirect {
    let business = state.db.get_business();
    let state_code = business.as_ref().map(|b| b.state.clone()).unwrap_or("VIC".to_string());
    let today = chrono::Local::now().date_naive();

    let deadline = state.db.get_pay_run(pay_run_id)
        .and_then(|r| NaiveDate::parse_from_str(&r.super_deadline, "%Y-%m-%d").ok())
        .unwrap_or(today);

    let paid = NaiveDate::parse_from_str(&form.payment_date, "%Y-%m-%d").ok();
    let status = determine_status(deadline, paid, &state_code);

    let payment = SuperPayment {
        id: 0,
        pay_run_id,
        payment_date: form.payment_date,
        clearing_house_receipt: form.clearing_house_receipt,
        amount_paid: form.amount_paid,
        fund_receipt_confirmed: false,
        fund_receipt_date: None,
        status: status.label().to_string(),
    };
    state.db.create_payment(&payment).ok();
    Redirect::to(&format!("/pay-runs/{}", pay_run_id))
}
