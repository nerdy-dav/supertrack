use axum::extract::State;
use axum::response::{Html, Redirect};
use axum::Form;
use std::sync::Arc;
use minijinja::context;
use serde::Deserialize;

use crate::AppState;
use crate::db::Employee;

#[derive(Deserialize)]
pub struct EmployeeForm {
    pub name: String,
    pub employment_type: String,
    pub super_fund_name: String,
    pub super_fund_usi: Option<String>,
    pub member_number: Option<String>,
    pub ote_weekly: f64,
    pub start_date: String,
}

pub async fn list(State(state): State<Arc<AppState>>) -> Html<String> {
    let employees = state.db.list_employees();
    let employees: Vec<serde_json::Value> = employees.iter().map(|e| {
        serde_json::json!({
            "id": e.id,
            "name": e.name,
            "employment_type": e.employment_type,
            "super_fund_name": e.super_fund_name,
            "member_number": e.member_number,
            "ote_weekly": e.ote_weekly,
            "super_per_week": format!("{:.2}", e.ote_weekly * 0.12),
            "active": e.active,
        })
    }).collect();
    let ctx = context! { employees => employees };
    crate::routes::render(&state.env, "employees.html", ctx)
}

pub async fn new_form(State(state): State<Arc<AppState>>) -> Html<String> {
    let ctx = context! {};
    crate::routes::render(&state.env, "employee_form.html", ctx)
}

pub async fn create(State(state): State<Arc<AppState>>, Form(form): Form<EmployeeForm>) -> Redirect {
    let employee = Employee {
        id: 0,
        name: form.name,
        employment_type: form.employment_type,
        super_fund_name: form.super_fund_name,
        super_fund_usi: form.super_fund_usi,
        member_number: form.member_number,
        ote_weekly: form.ote_weekly,
        active: true,
        start_date: form.start_date,
    };
    state.db.create_employee(&employee).ok();
    Redirect::to("/employees")
}


