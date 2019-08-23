//! Implements web server.

use crate::db::Db;
use crate::settings::Settings;
use crate::templates::*;
use crate::threading::ArcMutex;
use chrono::prelude::*;
use chrono::Duration;
use rouille::{router, Response};
use serde_json::json;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub const FAVICON: &[u8] = include_bytes!("statics/favicon.ico");
pub const FAVICON_16: &[u8] = include_bytes!("statics/favicon-16x16.png");
pub const FAVICON_32: &[u8] = include_bytes!("statics/favicon-32x32.png");
pub const APPLE_TOUCH_ICON: &[u8] = include_bytes!("statics/apple-touch-icon.png");
pub const ANDROID_CHROME_192: &[u8] = include_bytes!("statics/android-chrome-192x192.png");
pub const ANDROID_CHROME_512: &[u8] = include_bytes!("statics/android-chrome-512x512.png");

/// Start the web application.
pub fn start_server(settings: Settings, db: ArcMutex<Db>) -> ! {
    rouille::start_server(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), settings.http_port),
        move |request| {
            router!(request,
                (GET) ["/"] => index(&db),
                (GET) ["/sensors/{sensor}", sensor: String] => get_sensor(&db, &sensor),
                (GET) ["/sensors/{sensor}/json", sensor: String] => get_sensor_json(&db, &sensor),
                (GET) ["/favicon.ico"] => Response::from_data("image/x-icon", FAVICON.to_vec()),
                (GET) ["/apple-touch-icon.png"] => Response::from_data("image/png", APPLE_TOUCH_ICON.to_vec()),
                (GET) ["/favicon-32x32.png"] => Response::from_data("image/png", FAVICON_32.to_vec()),
                (GET) ["/favicon-16x16.png"] => Response::from_data("image/png", FAVICON_16.to_vec()),
                _ => Response::empty_404(),
            )
        },
    )
}

/// Get index page response.
fn index(db: &ArcMutex<Db>) -> Response {
    let readings = { db.lock().unwrap().select_latest_readings().unwrap() };
    Response::html(
        base::Base {
            body: Box::new(index::Index { readings }),
        }
        .to_string(),
    )
}

/// Get sensor page response.
fn get_sensor(db: &ArcMutex<Db>, sensor: &str) -> Response {
    let last = db.lock().unwrap().select_last_reading(&sensor).unwrap();
    let readings = db
        .lock()
        .unwrap()
        .select_readings(&sensor, &(Local::now() - Duration::minutes(5)))
        .unwrap();
    match last {
        Some(reading) => Response::html(
            base::Base {
                body: Box::new(sensor::Sensor {
                    last: reading,
                    readings,
                }),
            }
            .to_string(),
        ),
        None => Response::empty_404(),
    }
}

/// Get last sensor value JSON response.
fn get_sensor_json(db: &ArcMutex<Db>, sensor: &str) -> Response {
    match db.lock().unwrap().select_last_reading(&sensor).unwrap() {
        Some(reading) => Response::json(&json!({
            "value": &reading.value,
            "timestamp": &reading.timestamp,
        })),
        None => Response::empty_404(),
    }
}
