use crate::db::Db;
use crate::templates::*;
use rouille::router;
use std::sync::{Arc, Mutex};
use typed_html::dom::DOMTree;
use typed_html::html;

/// Start the web application.
pub fn start_server(db: Arc<Mutex<Db>>) {
    rouille::start_server("127.0.0.1:8080", move |request| {
        router!(request,
            (GET) (/) => {
                let measurements = {
                    db.lock().unwrap().select_latest_measurements()
                };
                // IndexTemplate { measurements }.into()
                let doc: DOMTree<String> = html!(
                    <p>"Hello Kitty"</p>
                );
                rouille::Response::html(doc.to_string())
            },
            _ => rouille::Response::empty_404(),
        )
    });
}
