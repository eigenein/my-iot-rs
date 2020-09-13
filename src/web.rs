//! Implements the web server.

use std::io::Cursor;

use chrono::Duration;
use itertools::Itertools;
use rocket::config::Environment;
use rocket::http::hyper::header::ETag;
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::response::content::Content;
use rocket::response::Redirect;
use rocket::{delete, get, routes, uri, Config, Response, Rocket, State};
use rocket_contrib::json::Json;

use crate::prelude::*;
use crate::settings::Settings;
use crate::web::cached_content::Cached;
use crate::web::if_none_match::IfNoneMatch;
use crate::web::to_html_string::ToHtmlString;
use std::convert::TryInto;

mod cached_content;
mod entity_tag;
mod if_none_match;
mod templates;
mod to_html_string;

const STATIC_MAX_AGE_SECS: u32 = 3600;

/// Start the web application.
pub fn start_server(settings: &Settings, db: Connection) -> Result {
    info!("Starting web server on port {}…", settings.http.port);
    Err(make_rocket(settings, db)?.launch().into())
}

/// Builds the [Rocket](https://rocket.rs/) application.
fn make_rocket(settings: &Settings, db: Connection) -> Result<Rocket> {
    Ok(rocket::custom(
        Config::build(Environment::Production)
            .port(settings.http.port)
            .keep_alive(600)
            .finalize()?,
    )
    .manage(db)
    .manage(settings.clone())
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
fn get_index(db: State<Connection>) -> Result<ToHtmlString<impl ToString>> {
    let actuals = task::block_on(db.select_actuals())?
        .into_iter()
        .group_by(|(sensor, _)| sensor.location.clone())
        .into_iter()
        .map(|(location, group)| (location, group.collect_vec()))
        .collect_vec();
    Ok(ToHtmlString(templates::IndexTemplate { actuals }))
}

#[get("/settings")]
fn get_settings(settings: State<Settings>) -> Result<ToHtmlString<impl ToString>> {
    Ok(ToHtmlString(templates::SettingsTemplate {
        settings: toml::to_string_pretty(&toml::Value::try_from(settings.inner())?)?,
    }))
}

#[get("/sensors/<sensor_id>?<minutes>")]
fn get_sensor<'r>(
    db: State<Connection>,
    if_none_match: Option<IfNoneMatch>,
    sensor_id: String,
    minutes: Option<i64>,
) -> Result<Response<'r>> {
    if let Some((sensor, reading)) = task::block_on(db.select_sensor(&sensor_id))? {
        if let Some(IfNoneMatch(entity_tag)) = if_none_match {
            if reading.entity_tag().weak_eq(&entity_tag) {
                // If there's a match, we can avoid spending CPU on generation of the chart.
                return Response::build().status(Status::NotModified).ok();
            }
        }

        let minutes = minutes.unwrap_or(60);
        let readings = task::block_on(db.select_readings(&sensor_id, &(Local::now() - Duration::minutes(minutes))))?;
        let chart = if readings.is_empty() {
            // language=html
            r#"<div class="notification content"><p>No data points within the period.</p></div>"#.to_string()
        } else if TryInto::<f64>::try_into(&reading.value).is_ok() {
            templates::F64ChartPartialTemplate::new(
                &sensor.title(),
                readings,
                if let Value::Energy(_) = reading.value {
                    WH_IN_JOULE
                } else {
                    1.0
                },
            )
            .to_string()
        } else {
            // language=html
            r#"<div class="notification content"><p>Chart is unimplemented for this sensor.</p></div>"#.to_string()
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
                    reading_count: task::block_on(db.select_sensor_reading_count(&sensor_id))?,
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
    task::block_on(db.delete_sensor(&sensor_id))?;
    Ok(Redirect::to(uri!(get_index)))
}

#[get("/sensors/<sensor_id>/json")]
fn get_sensor_json(db: State<Connection>, sensor_id: String) -> Result<Option<Json<Reading>>> {
    // TODO: ETag
    // TODO: Cache-Control: private, no-cache
    Ok(task::block_on(db.select_sensor(&sensor_id))?.map(|(_, reading)| Json(reading)))
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
    use rocket::local::Client;

    use crate::settings::*;

    use super::*;

    #[async_std::test]
    async fn index_ok() -> Result {
        let client = client().await?;
        let response = client.get("/").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::HTML));
        Ok(())
    }

    #[async_std::test]
    async fn settings_ok() -> Result {
        let client = client().await?;
        let response = client.get("/settings").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::HTML));
        Ok(())
    }

    #[async_std::test]
    async fn favicon_ok() -> Result {
        let client = client().await?;
        let response = client.get("/favicon.ico").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::Icon));
        Ok(())
    }

    async fn client() -> crate::Result<Client> {
        Ok(Client::new(make_rocket(
            &toml::from_str::<Settings>("")?,
            Connection::open(":memory:").await?,
        )?)?)
    }
}
