//! Implements the web server.

use crate::prelude::*;
use crate::settings::Settings;
use chrono::Duration;
use itertools::Itertools;
use rocket::config::Environment;
use rocket::http::ContentType;
use rocket::response::content::{Content, Html};
use rocket::{get, routes, Config, Rocket, State};
use rocket_contrib::json::Json;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

mod templates;

/// Start the web application.
pub fn start_server(settings: &Settings, db: Connection, message_counter: Arc<AtomicU64>) -> Result<()> {
    Err(Box::new(make_rocket(settings, db, message_counter)?.launch()))
}

/// Builds the [Rocket](https://rocket.rs/) application.
fn make_rocket(settings: &Settings, db: Connection, message_counter: Arc<AtomicU64>) -> Result<Rocket> {
    Ok(rocket::custom(
        Config::build(Environment::Production)
            .port(settings.http_port)
            .keep_alive(600)
            .finalize()?,
    )
    .manage(db)
    .manage(settings.clone())
    .manage(MessageCounter(message_counter))
    .mount(
        "/",
        routes![
            get_index,
            get_settings,
            get_sensor,
            get_sensor_json,
            get_favicon,
            get_favicon_16,
            get_favicon_32,
            get_apple_touch_icon,
            get_android_chrome_192,
            get_android_chrome_512,
            get_bulma_css,
            get_bulma_prefers_dark,
            get_chart_js,
            get_font_awesome,
            get_webfonts_fa_solid_900,
            get_webfonts_fa_regular_400,
            get_webfonts_fa_brands_400,
        ],
    ))
}

#[get("/")]
fn get_index(db: State<Connection>, message_counter: State<MessageCounter>) -> Result<Html<String>> {
    let actuals = db
        .select_actuals()?
        .into_iter()
        .group_by(|(sensor, _)| sensor.room_title.clone())
        .into_iter()
        .map(|(room_title, group)| (room_title, group.collect_vec()))
        .collect_vec();
    Ok(Html(
        templates::IndexTemplate {
            actuals,
            message_count: message_counter.inner().value(),
        }
        .to_string(),
    ))
}

#[get("/settings")]
fn get_settings(settings: State<Settings>, message_counter: State<MessageCounter>) -> Result<Html<String>> {
    Ok(Html(
        templates::SettingsTemplate {
            settings: toml::to_string_pretty(&toml::Value::try_from(settings.inner())?)?,
            message_count: message_counter.inner().value(),
        }
        .to_string(),
    ))
}

#[get("/sensors/<sensor_id>?<minutes>")]
fn get_sensor(
    db: State<Connection>,
    sensor_id: String,
    minutes: Option<i64>,
    message_counter: State<MessageCounter>,
) -> Result<Option<Html<String>>> {
    let period = Duration::minutes(minutes.unwrap_or(60));
    if let Some((sensor, reading)) = db.select_sensor(&sensor_id)? {
        let chart = match reading.value {
            Value::Temperature(_)
            | Value::Duration(_)
            | Value::Energy(_)
            | Value::Length(_)
            | Value::Power(_)
            | Value::RelativeIntensity(_)
            | Value::Rh(_)
            | Value::Volume(_) => templates::F64ChartPartialTemplate::new(
                &sensor.title(),
                db.select_values(&sensor_id, &(Local::now() - period))?,
            )
            .to_string(),
            _ => "".into(),
        };

        Ok(Some(Html(
            templates::SensorTemplate {
                sensor,
                reading,
                chart,
                message_count: message_counter.inner().value(),
            }
            .to_string(),
        )))
    } else {
        Ok(None)
    }
}

#[get("/sensors/<sensor_id>/json")]
fn get_sensor_json(db: State<Connection>, sensor_id: String) -> Result<Option<Json<Reading>>> {
    Ok(db.select_sensor(&sensor_id)?.map(|(_, reading)| Json(reading)))
}

#[get("/favicon.ico")]
fn get_favicon() -> Content<&'static [u8]> {
    Content(ContentType::Icon, include_bytes!("statics/favicon.ico"))
}

#[get("/static/favicon-16x16.png")]
fn get_favicon_16() -> Content<&'static [u8]> {
    Content(ContentType::PNG, include_bytes!("statics/favicon-16x16.png"))
}

#[get("/static/favicon-32x32.png")]
fn get_favicon_32() -> Content<&'static [u8]> {
    Content(ContentType::PNG, include_bytes!("statics/favicon-32x32.png"))
}

#[get("/static/apple-touch-icon.png")]
fn get_apple_touch_icon() -> Content<&'static [u8]> {
    Content(ContentType::PNG, include_bytes!("statics/apple-touch-icon.png"))
}

#[get("/static/android-chrome-192x192.png")]
fn get_android_chrome_192() -> Content<&'static [u8]> {
    Content(ContentType::PNG, include_bytes!("statics/android-chrome-192x192.png"))
}

#[get("/static/android-chrome-512x512.png")]
fn get_android_chrome_512() -> Content<&'static [u8]> {
    Content(ContentType::PNG, include_bytes!("statics/android-chrome-512x512.png"))
}

#[get("/static/bulma.min.css")]
fn get_bulma_css() -> Content<&'static [u8]> {
    Content(ContentType::CSS, include_bytes!("statics/bulma.min.css"))
}

#[get("/static/bulma-prefers-dark.css")]
fn get_bulma_prefers_dark() -> Content<&'static [u8]> {
    Content(ContentType::CSS, include_bytes!("statics/bulma-prefers-dark.css"))
}

#[get("/static/Chart.bundle.min.js")]
fn get_chart_js() -> Content<&'static [u8]> {
    Content(ContentType::JavaScript, include_bytes!("statics/Chart.bundle.min.js"))
}

#[get("/static/fontawesome.css")]
fn get_font_awesome() -> Content<&'static [u8]> {
    Content(
        ContentType::CSS,
        include_bytes!("statics/fontawesome-free-5.13.1-web/css/all.css"),
    )
}

#[get("/webfonts/fa-solid-900.woff2")]
fn get_webfonts_fa_solid_900() -> Content<&'static [u8]> {
    Content(
        ContentType::WOFF2,
        include_bytes!("statics/fontawesome-free-5.13.1-web/webfonts/fa-solid-900.woff2"),
    )
}

#[get("/webfonts/fa-regular-400.woff2")]
fn get_webfonts_fa_regular_400() -> Content<&'static [u8]> {
    Content(
        ContentType::WOFF2,
        include_bytes!("statics/fontawesome-free-5.13.1-web/webfonts/fa-regular-400.woff2"),
    )
}

#[get("/webfonts/fa-brands-400.woff2")]
fn get_webfonts_fa_brands_400() -> Content<&'static [u8]> {
    Content(
        ContentType::WOFF2,
        include_bytes!("statics/fontawesome-free-5.13.1-web/webfonts/fa-brands-400.woff2"),
    )
}

struct MessageCounter(Arc<AtomicU64>);

impl MessageCounter {
    pub fn value(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::*;
    use rocket::http::Status;
    use rocket::local::Client;

    type Result = crate::Result<()>;

    #[test]
    fn index_ok() -> Result {
        let client = client()?;
        let response = client.get("/").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::HTML));
        Ok(())
    }

    #[test]
    fn settings_ok() -> Result {
        let client = client()?;
        let response = client.get("/settings").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::HTML));
        Ok(())
    }

    #[test]
    fn favicon_ok() -> Result {
        let client = client()?;
        let response = client.get("/favicon.ico").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::Icon));
        Ok(())
    }

    fn client() -> crate::Result<Client> {
        Ok(Client::new(make_rocket(
            &Settings {
                http_port: default_http_port(),
                services: HashMap::new(),
            },
            Connection::open_and_initialize(":memory:")?,
            Arc::new(AtomicU64::new(0)),
        )?)?)
    }
}
