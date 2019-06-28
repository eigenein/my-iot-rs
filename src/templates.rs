//! Describes all the templates.

use crate::measurement::Measurement;
use askama::Template;
use rouille::Response;

/// Index page template.
#[derive(Debug, askama::Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub measurements: Vec<Measurement>,
}

macro_rules! into_response {
    ($t:ty) => {
        impl Into<Response> for $t {
            fn into(self) -> Response {
                Response::html(self.render().unwrap())
            }
        }
    };
}

into_response!(IndexTemplate);
