use crate::templates::*;
use askama::Template;
use rouille::router;

/// Start the web application.
pub fn start_server() {
    rouille::start_server("127.0.0.1:8080", move |request| {
        router!(request,
            (GET) (/) => {
                rouille::Response::html(IndexTemplate {}.render().unwrap())
            },
            _ => rouille::Response::empty_404(),
        )
    });
}
