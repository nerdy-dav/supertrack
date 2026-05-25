use axum::extract::State;
use axum::response::{Html, Redirect};
use axum::Form;
use std::sync::Arc;
use minijinja::context;
use serde::Deserialize;

use crate::AppState;
use crate::db::Business;

#[derive(Deserialize)]
pub struct SetupForm {
    pub name: String,
    pub abn: String,
    pub state: String,
    pub pay_frequency: String,
    pub payroll_software: Option<String>,
    pub clearing_house: Option<String>,
    pub clearing_house_lag_days: Option<i64>,
    pub sgc_rate: Option<f64>,
}

pub async fn show(State(state): State<Arc<AppState>>) -> Html<String> {
    let business = state.db.get_business();
    let ctx = context! {
        business => business,
        states => vec!["NSW", "VIC", "QLD", "SA", "WA", "TAS", "ACT", "NT"],
        frequencies => vec!["weekly", "fortnightly", "monthly"],
    };
    crate::routes::render(&state.env, "setup.html", ctx)
}

pub async fn save(State(state): State<Arc<AppState>>, Form(form): Form<SetupForm>) -> Redirect {
    let business = Business {
        id: 1,
        name: form.name,
        abn: form.abn,
        state: form.state,
        pay_frequency: form.pay_frequency,
        payroll_software: form.payroll_software,
        clearing_house: form.clearing_house,
        clearing_house_lag_days: form.clearing_house_lag_days.unwrap_or(1),
        sgc_rate: form.sgc_rate.unwrap_or(0.12),
    };
    state.db.upsert_business(&business).ok();
    Redirect::to("/")
}
