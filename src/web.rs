//! Web app functionality.

use crate::templates::*;
use actix_web::{get, HttpResponse};
use askama::Template;

/// Index page.
#[get("/")]
pub fn index() -> actix_web::Result<HttpResponse> {
    render(&IndexTemplate {})
}

/// Render a template.
/// Ideally, this should be implemented as a trait but I didn't manage to do so.
/// See also https://github.com/actix/examples/blob/b806d2254dff2e0db8a22141957a848ecbef77c3/template_askama/src/main.rs
fn render<T: Template>(t: &T) -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(t.render().unwrap()))
}
