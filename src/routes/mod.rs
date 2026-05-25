pub mod dashboard;
pub mod setup;
pub mod employees;
pub mod pay_runs;
pub mod payments;
pub mod calculator;
pub mod reports;

use axum::response::Html;

pub fn render(env: &minijinja::Environment, template: &str, ctx: minijinja::Value) -> Html<String> {
    match env.get_template(template) {
        Ok(tmpl) => match tmpl.render(ctx) {
            Ok(html) => Html(html),
            Err(e) => Html(format!("<pre>Render error in {}: {}</pre>", template, e)),
        },
        Err(e) => Html(format!("<pre>Template not found {}: {}</pre>", template, e)),
    }
}
