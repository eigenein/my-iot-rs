//! Implements web server.

use crate::consts;
use crate::db::Db;
use crate::settings::Settings;
use crate::templates::*;
use crate::types::ArcMutex;
use chrono::prelude::*;
use chrono::Duration;
use rouille::{router, Response};
use serde_json::json;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Start the web application.
pub fn start_server(settings: Settings, db: ArcMutex<Db>) -> ! {
    rouille::start_server(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), settings.http_port.unwrap_or(8081)),
        move |request| {
            router!(request,
                (GET) ["/"] => index(&db),
                (GET) ["/sensors/{sensor}", sensor: String] => get_sensor(&db, &sensor),
                (GET) ["/sensors/{sensor}/json", sensor: String] => get_sensor_json(&db, &sensor),
                (GET) ["/status"] => get_status(&settings),
                (GET) ["/favicon.ico"] => Response::from_data("image/x-icon", consts::FAVICON.to_vec()),
                (GET) ["/apple-touch-icon.png"] => Response::from_data("image/png", consts::APPLE_TOUCH_ICON.to_vec()),
                (GET) ["/favicon-32x32.png"] => Response::from_data("image/png", consts::FAVICON_32.to_vec()),
                (GET) ["/favicon-16x16.png"] => Response::from_data("image/png", consts::FAVICON_16.to_vec()),
                _ => Response::empty_404(),
            )
        },
    )
}

/// Get index page response.
fn index(db: &ArcMutex<Db>) -> Response {
    let readings = { db.lock().unwrap().select_latest_readings() };
    Response::html(
        base::Base {
            body: Box::new(index::Index { readings }),
        }
        .to_string(),
    )
}

/// Get sensor page response.
fn get_sensor(db: &ArcMutex<Db>, sensor: &str) -> Response {
    let (last, readings) = {
        let db = db.lock().unwrap();
        (
            db.select_last_reading(&sensor),
            db.select_readings(&sensor, &(Local::now() - Duration::minutes(5))),
        )
    };
    match last {
        Some(reading) => Response::html(
            base::Base {
                body: Box::new(sensor::Sensor { last: reading, readings }),
            }
            .to_string(),
        ),
        None => Response::empty_404(),
    }
}

/// Get last sensor value JSON response.
fn get_sensor_json(db: &ArcMutex<Db>, sensor: &str) -> Response {
    match db.lock().unwrap().select_last_reading(&sensor) {
        Some(reading) => Response::json(&json!({
            "value": &reading.value,
            "timestamp": &reading.timestamp,
        })),
        None => Response::empty_404(),
    }
}

/// Get status page response.
fn get_status(settings: &Settings) -> Response {
    let settings = settings.clone();
    Response::html(
        base::Base {
            body: Box::new(status::Status { settings }),
        }
        .to_string(),
    )
}
