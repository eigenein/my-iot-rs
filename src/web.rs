//! Implements web server.

use crate::prelude::*;
use crate::settings::Settings;
use crate::templates;
use lazy_static::lazy_static;
use rouille::{router, Response};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};

const FAVICON: &[u8] = include_bytes!("statics/favicon.ico");

/// Start the web application.
pub fn start_server(settings: Settings, db: Arc<Mutex<Connection>>) -> ! {
    rouille::start_server(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), settings.http_port),
        move |request| {
            router!(request,
                (GET) ["/"] => index(&db, &settings),
                (GET) ["/sensors/{sensor_id}", sensor_id: String] => get_sensor(&db, &sensor_id),
                (GET) ["/sensors/{sensor_id}/json", sensor_id: String] => get_sensor_json(&db, &sensor_id),
                (GET) ["/favicon.ico"] => Response::from_data("image/x-icon", FAVICON.to_vec()),
                (GET) ["/static/{key}", key: String] => match STATICS.get(key.as_str()) {
                    Some((content_type, data)) => Response::from_data(content_type, data.clone()),
                    None => Response::empty_404(),
                },
                _ => Response::empty_404(),
            )
        },
    )
}

/// Get index page response.
fn index(db: &Arc<Mutex<Connection>>, settings: &Settings) -> Response {
    Response::html(
        templates::IndexTemplate::new(&db.lock().unwrap(), settings.max_sensor_age_ms)
            .unwrap()
            .to_string(),
    )
}

/// Get sensor page response.
fn get_sensor(db: &Arc<Mutex<Connection>>, sensor_id: &str) -> Response {
    let actual = db.lock().unwrap().get_sensor(&sensor_id).unwrap();
    match actual {
        Some((sensor, reading)) => Response::html(templates::SensorTemplate::new(sensor, reading).to_string()),
        None => Response::empty_404(),
    }
}

/// Get last sensor value JSON response.
fn get_sensor_json(db: &Arc<Mutex<Connection>>, sensor_id: &str) -> Response {
    match db.lock().unwrap().get_sensor(&sensor_id).unwrap() {
        Some((_, reading)) => Response::json(&reading),
        None => Response::empty_404(),
    }
}

lazy_static! {
    static ref STATICS: HashMap<&'static str, (String, Vec<u8>)> = {
        let mut map = HashMap::new();
        map.insert(
            "favicon-16x16.png",
            ("image/png".into(), include_bytes!("statics/favicon-16x16.png").to_vec()),
        );
        map.insert(
            "favicon-32x32.png",
            ("image/png".into(), include_bytes!("statics/favicon-32x32.png").to_vec()),
        );
        map.insert(
            "apple-touch-icon.png",
            (
                "image/png".into(),
                include_bytes!("statics/apple-touch-icon.png").to_vec(),
            ),
        );
        map.insert(
            "android-chrome-192x192.png",
            (
                "image/png".into(),
                include_bytes!("statics/android-chrome-192x192.png").to_vec(),
            ),
        );
        map.insert(
            "android-chrome-512x512.png",
            (
                "image/png".into(),
                include_bytes!("statics/android-chrome-512x512.png").to_vec(),
            ),
        );
        map.insert(
            "bulma.min.css",
            ("text/css".into(), include_bytes!("statics/bulma.min.css").to_vec()),
        );
        map.insert(
            "bulma-prefers-dark.css",
            (
                "text/css".into(),
                include_bytes!("statics/bulma-prefers-dark.css").to_vec(),
            ),
        );
        map.insert(
            "plotly-1.5.0.min.js",
            (
                "application/javascript".into(),
                include_bytes!("statics/plotly-1.5.0.min.js").to_vec(),
            ),
        );
        map
    };
}
