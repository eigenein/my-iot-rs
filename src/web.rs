use crate::db::Db;
use crate::templates::*;
use rouille::{router, Response};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};

/// Start the web application.
pub fn start_server(port: u16, db: Arc<Mutex<Db>>) {
    rouille::start_server(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port),
        move |request| {
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
        },
    );
}
