use axum::extract::{State, Query};
use axum::response::Html;
use std::sync::Arc;
use minijinja::context;
use serde::Deserialize;

use crate::AppState;
use crate::domain::cashflow::calculate_impact;

#[derive(Deserialize)]
pub struct CalcQuery {
    pub monthly_wages: Option<f64>,
}

pub async fn show(State(state): State<Arc<AppState>>, Query(q): Query<CalcQuery>) -> Html<String> {
    let business = state.db.get_business();
    let pay_freq = business.as_ref().map(|b| b.pay_frequency.clone()).unwrap_or("fortnightly".to_string());
    let sgc_rate = business.as_ref().map(|b| b.sgc_rate).unwrap_or(0.12);

    if let Some(wages) = q.monthly_wages {
        let impact = calculate_impact(wages, &pay_freq, sgc_rate);
        let ctx = context! { impact => impact, monthly_wages => wages, pay_freq => pay_freq };
        return crate::routes::render(&state.env, "calculator.html", ctx);
    }
    crate::routes::render(&state.env, "calculator.html", context! {})
}
