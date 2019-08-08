//! Implements web server.

use crate::db::Db;
use crate::statics;
use crate::templates::*;
use chrono::prelude::*;
use chrono::Duration;
use rouille::{router, Response};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};

/// Start the web application.
pub fn start_server(port: u16, db: Arc<Mutex<Db>>) {
    rouille::start_server(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port),
        move |request| {
            router!(request,
                (GET) ["/"] => {
                    let measurements = {
                        db.lock().unwrap().select_latest_measurements()
                    };
                    Response::html(base::Base {
                        body: Box::new(index::Index { measurements }),
                    }.to_string())
                },
                (GET) ["/sensors/{sensor}", sensor: String] => {
                    let (last, _measurements) = {
                        db.lock().unwrap().select_sensor_measurements(&sensor, &(Local::now() - Duration::minutes(5)))
                    };
                    Response::html(base::Base {
                        body: Box::new(sensor::Sensor { last }),
                    }.to_string())
                },
                (GET) ["/services"] => {
                    Response::html(base::Base {
                        body: Box::new(services::Services { }),
                    }.to_string())
                },
                (GET) ["/favicon.ico"] => Response::from_data("image/x-icon", statics::FAVICON.to_vec()),
                (GET) ["/apple-touch-icon.png"] => Response::from_data("image/png", statics::APPLE_TOUCH_ICON.to_vec()),
                (GET) ["/favicon-32x32.png"] => Response::from_data("image/png", statics::FAVICON_32.to_vec()),
                (GET) ["/favicon-16x16.png"] => Response::from_data("image/png", statics::FAVICON_16.to_vec()),
                _ => Response::empty_404(),
            )
        },
    );
}
