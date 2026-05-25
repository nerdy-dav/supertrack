mod db;
mod domain;
mod routes;

use axum::{
    Router,
    routing::{get, post},
    response::Response,
    body::Body,
    http::header,
};
use std::sync::Arc;

pub struct AppState {
    pub db: Arc<db::Database>,
    pub env: Arc<minijinja::Environment<'static>>,
    pub db_path: String,
}

async fn serve_css() -> Response<Body> {
    let css = include_str!("../static/style.css");
    Response::builder()
        .header(header::CONTENT_TYPE, "text/css")
        .body(Body::from(css))
        .unwrap()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let db_path = "supertrack.db".to_string();
    let database = db::Database::new(&db_path)?;
    database.migrate()?;

    let mut env = minijinja::Environment::new();
    env.set_loader(minijinja::path_loader("templates"));
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Chainable);

    let state = Arc::new(AppState {
        db: Arc::new(database),
        env: Arc::new(env),
        db_path,
    });

    let app = Router::new()
        .route("/static/style.css", get(serve_css))
        .route("/", get(routes::dashboard::index))
        .route("/setup", get(routes::setup::show).post(routes::setup::save))
        .route("/employees", get(routes::employees::list))
        .route("/employees/new", get(routes::employees::new_form).post(routes::employees::create))
        .route("/pay-runs", get(routes::pay_runs::list))
        .route("/pay-runs/new", get(routes::pay_runs::new_form).post(routes::pay_runs::create))
        .route("/pay-runs/:id", get(routes::pay_runs::show))
        .route("/payments/new/:pay_run_id", get(routes::payments::new_form).post(routes::payments::create))
        .route("/calculator", get(routes::calculator::show))
        .route("/reports", get(routes::reports::index))
        .route("/reports/export.csv", get(routes::reports::export_csv))
        .route("/backup", get(routes::backup::download))
        .route("/backup/restore", post(routes::backup::restore))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    println!("\n  SuperTrack AU — Payday Super Compliance Manager");
    println!("  Open http://localhost:3000 in your browser\n");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
