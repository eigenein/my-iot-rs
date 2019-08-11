//! Implements web server.

use crate::db::Db;
use crate::settings::Settings;
use crate::statics;
use crate::templates::*;
use crate::types::ArcMutex;
use chrono::prelude::*;
use chrono::Duration;
use rouille::{router, Response};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Start the web application.
pub fn start_server(settings: Settings, db: ArcMutex<Db>) -> ! {
    rouille::start_server(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), settings.http_port.unwrap_or(8081)),
        move |request| {
            router!(request,
                (GET) ["/"] => {
                    let readings = {
                        db.lock().unwrap().select_latest_readings()
                    };
                    Response::html(base::Base {
                        body: Box::new(index::Index { readings }),
                    }.to_string())
                },
                (GET) ["/sensors/{sensor}", sensor: String] => {
                    let (last, _readings) = {
                        db.lock().unwrap().select_sensor_readings(&sensor, &(Local::now() - Duration::minutes(5)))
                    };
                    Response::html(base::Base {
                        body: Box::new(sensor::Sensor { last }),
                    }.to_string())
                },
                (GET) ["/status"] => {
                    let settings = settings.clone();
                    Response::html(base::Base {
                        body: Box::new(status::Status { settings: settings }),
                    }.to_string())
                },
                (GET) ["/favicon.ico"] => Response::from_data("image/x-icon", statics::FAVICON.to_vec()),
                (GET) ["/apple-touch-icon.png"] => Response::from_data("image/png", statics::APPLE_TOUCH_ICON.to_vec()),
                (GET) ["/favicon-32x32.png"] => Response::from_data("image/png", statics::FAVICON_32.to_vec()),
                (GET) ["/favicon-16x16.png"] => Response::from_data("image/png", statics::FAVICON_16.to_vec()),
                _ => Response::empty_404(),
            )
        },
    )
}
