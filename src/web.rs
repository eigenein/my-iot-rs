use crate::db::Db;
use crate::templates::*;
use rouille::{router, Response};
use std::sync::{Arc, Mutex};

/// Start the web application.
pub fn start_server(db: Arc<Mutex<Db>>) {
    rouille::start_server("127.0.0.1:8080", move |request| {
        router!(request,
            (GET) (/) => {
                let measurements = {
                    db.lock().unwrap().select_latest_measurements()
                };
                Response::html(Base {
                    body: Box::new(Index { measurements }),
                }.to_string())
            },
            _ => Response::empty_404(),
        )
    });
}
