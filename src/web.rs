//! Implements the web server.

use crate::prelude::*;
use crate::settings::Settings;
use crate::web::cached_content::Cached;
use crate::web::if_none_match::IfNoneMatch;
use crate::web::message_counter::MessageCounter;
use chrono::Duration;
use itertools::Itertools;
use rocket::config::Environment;
use rocket::http::hyper::header::ETag;
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::response::content::{Content, Html};
use rocket::response::Redirect;
use rocket::{delete, get, routes, uri, Config, Response, Rocket, State};
use rocket_contrib::json::Json;
use std::io::Cursor;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

mod cached_content;
mod entity_tag;
mod if_none_match;
mod message_counter;
mod templates;

const STATIC_MAX_AGE_SECS: u32 = 3600;

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
            delete_sensor,
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
            get_sw_js,
            get_webmanifest,
        ],
    ))
}

#[get("/")]
fn get_index(db: State<Connection>, message_counter: State<MessageCounter>) -> Result<Html<String>> {
    // TODO: `ETag`.
    let actuals = db
        .select_actuals()?
        .into_iter()
        .group_by(|(sensor, _)| sensor.location.clone())
        .into_iter()
        .map(|(location, group)| (location, group.collect_vec()))
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
fn get_sensor<'r>(
    db: State<Connection>,
    message_counter: State<MessageCounter>,
    if_none_match: Option<IfNoneMatch>,
    sensor_id: String,
    minutes: Option<i64>,
) -> Result<Response<'r>> {
    if let Some((sensor, reading)) = db.select_sensor(&sensor_id)? {
        if let Some(IfNoneMatch(entity_tag)) = if_none_match {
            if reading.entity_tag().weak_eq(&entity_tag) {
                // If there's a match, we can avoid spending CPU on generation of the chart.
                return Response::build().status(Status::NotModified).ok();
            }
        }

        let minutes = minutes.unwrap_or(60);
        let chart = if reading.value.is_f64() {
            templates::F64ChartPartialTemplate::new(
                &sensor.title(),
                db.select_values(&sensor_id, &(Local::now() - Duration::minutes(minutes)))?,
                if let Value::Energy(_) = reading.value {
                    WH_IN_JOULE
                } else {
                    1.0
                },
            )
            .to_string()
        } else {
            "".into()
        };

        Response::build()
            .header(ContentType::HTML)
            .header(ETag(reading.entity_tag()))
            .sized_body(Cursor::new(
                templates::SensorTemplate {
                    sensor,
                    reading,
                    chart,
                    minutes,
                    reading_count: db.select_sensor_reading_count(&sensor_id)?,
                    message_count: message_counter.inner().value(),
                }
                .to_string(),
            ))
            .ok()
    } else {
        Response::build().status(Status::NotFound).ok()
    }
}

#[delete("/sensors/<sensor_id>")]
fn delete_sensor(db: State<Connection>, sensor_id: String) -> Result<Redirect> {
    db.delete_sensor(&sensor_id)?;
    Ok(Redirect::to(uri!(get_index)))
}

#[get("/sensors/<sensor_id>/json")]
fn get_sensor_json(db: State<Connection>, sensor_id: String) -> Result<Option<Json<Reading>>> {
    Ok(db.select_sensor(&sensor_id)?.map(|(_, reading)| Json(reading)))
}

#[get("/favicon.ico")]
fn get_favicon() -> Cached {
    Cached(
        STATIC_MAX_AGE_SECS,
        Content(ContentType::Icon, include_bytes!("statics/favicon.ico")),
    )
}

#[get("/static/favicon-16x16.png")]
fn get_favicon_16() -> Cached {
    Cached(
        STATIC_MAX_AGE_SECS,
        Content(ContentType::PNG, include_bytes!("statics/favicon-16x16.png")),
    )
}

#[get("/static/favicon-32x32.png")]
fn get_favicon_32() -> Cached {
    Cached(
        STATIC_MAX_AGE_SECS,
        Content(ContentType::PNG, include_bytes!("statics/favicon-32x32.png")),
    )
}

#[get("/static/apple-touch-icon.png")]
fn get_apple_touch_icon() -> Cached {
    Cached(
        STATIC_MAX_AGE_SECS,
        Content(ContentType::PNG, include_bytes!("statics/apple-touch-icon.png")),
    )
}

#[get("/static/android-chrome-192x192.png")]
fn get_android_chrome_192() -> Cached {
    Cached(
        STATIC_MAX_AGE_SECS,
        Content(ContentType::PNG, include_bytes!("statics/android-chrome-192x192.png")),
    )
}

#[get("/static/android-chrome-512x512.png")]
fn get_android_chrome_512() -> Cached {
    Cached(
        STATIC_MAX_AGE_SECS,
        Content(ContentType::PNG, include_bytes!("statics/android-chrome-512x512.png")),
    )
}

#[get("/static/bulma.min.css")]
fn get_bulma_css() -> Cached {
    Cached(
        STATIC_MAX_AGE_SECS,
        Content(ContentType::CSS, include_bytes!("statics/bulma.min.css")),
    )
}

#[get("/static/bulma-prefers-dark.css")]
fn get_bulma_prefers_dark() -> Cached {
    Cached(
        STATIC_MAX_AGE_SECS,
        Content(ContentType::CSS, include_bytes!("statics/bulma-prefers-dark.css")),
    )
}

#[get("/static/Chart.bundle.min.js")]
fn get_chart_js() -> Cached {
    Cached(
        STATIC_MAX_AGE_SECS,
        Content(ContentType::JavaScript, include_bytes!("statics/Chart.bundle.min.js")),
    )
}

#[get("/static/fontawesome.css")]
fn get_font_awesome() -> Cached {
    Cached(
        STATIC_MAX_AGE_SECS,
        Content(
            ContentType::CSS,
            include_bytes!("statics/fontawesome-free-5.13.1-web/css/all.css"),
        ),
    )
}

#[get("/webfonts/fa-solid-900.woff2")]
fn get_webfonts_fa_solid_900() -> Cached {
    Cached(
        STATIC_MAX_AGE_SECS,
        Content(
            ContentType::WOFF2,
            include_bytes!("statics/fontawesome-free-5.13.1-web/webfonts/fa-solid-900.woff2"),
        ),
    )
}

#[get("/webfonts/fa-regular-400.woff2")]
fn get_webfonts_fa_regular_400() -> Cached {
    Cached(
        STATIC_MAX_AGE_SECS,
        Content(
            ContentType::WOFF2,
            include_bytes!("statics/fontawesome-free-5.13.1-web/webfonts/fa-regular-400.woff2"),
        ),
    )
}

#[get("/webfonts/fa-brands-400.woff2")]
fn get_webfonts_fa_brands_400() -> Cached {
    Cached(
        STATIC_MAX_AGE_SECS,
        Content(
            ContentType::WOFF2,
            include_bytes!("statics/fontawesome-free-5.13.1-web/webfonts/fa-brands-400.woff2"),
        ),
    )
}

#[get("/sw.js")]
fn get_sw_js() -> Cached {
    Cached(
        STATIC_MAX_AGE_SECS,
        Content(ContentType::JavaScript, include_bytes!("statics/sw.js")),
    )
}

#[get("/my-iot.webmanifest")]
fn get_webmanifest() -> Cached {
    Cached(
        STATIC_MAX_AGE_SECS,
        Content(ContentType::JSON, include_bytes!("statics/my-iot.webmanifest")),
    )
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
