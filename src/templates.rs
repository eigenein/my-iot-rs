//! Describes all the templates.

/// Index page template.
#[derive(Debug, askama::Template)]
#[template(path = "index.html")]
pub struct IndexTemplate;
