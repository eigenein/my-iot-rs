//! Implements web server.

use crate::prelude::*;
use crate::settings::Settings;
use itertools::Itertools;
use lazy_static::lazy_static;
use rocket::config::Environment;
use rocket::http::ContentType;
use rocket::response::content::{Content, Html};
use rocket::{get, routes, Config, Rocket, State};
use rocket_contrib::json::Json;
use std::path::PathBuf;

mod templates;

const FAVICON: &[u8] = include_bytes!("statics/favicon.ico");

/// Start the web application.
pub fn start_server(settings: &Settings, db: Connection) -> Result<()> {
    Err(Box::new(make_rocket(settings, db)?.launch()))
}

fn make_rocket(settings: &Settings, db: Connection) -> Result<Rocket> {
    Ok(rocket::custom(
        Config::build(Environment::Production)
            .port(settings.http_port)
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
            get_favicon,
            get_static,
            get_webfonts,
            get_sensor,
            get_sensor_json,
        ],
    ))
}

#[get("/")]
fn get_index(db: State<Connection>) -> Result<Html<String>> {
    let actuals = db
        .select_actuals()?
        .into_iter()
        .group_by(|(sensor, _)| sensor.room_title.clone())
        .into_iter()
        .map(|(room_title, group)| (room_title, group.collect_vec()))
        .collect_vec();
    Ok(Html(templates::IndexTemplate { actuals }.to_string()))
}

#[get("/settings")]
fn get_settings(settings: State<Settings>) -> Result<Html<String>> {
    Ok(Html(templates::SettingsTemplate::new(&settings)?.to_string()))
}

#[get("/favicon.ico")]
fn get_favicon() -> Content<&'static [u8]> {
    Content(ContentType::Icon, FAVICON)
}

#[get("/static/<key..>")]
fn get_static(key: PathBuf) -> Option<Content<&'static [u8]>> {
    get_bundled_static(key)
}

#[get("/webfonts/<key..>")]
fn get_webfonts(key: PathBuf) -> Option<Content<&'static [u8]>> {
    get_bundled_static(key)
}

fn get_bundled_static(key: PathBuf) -> Option<Content<&'static [u8]>> {
    STATICS
        .get(&key)
        .map(|(content_type, content)| Content(content_type.clone(), &content[..]))
}

#[get("/sensors/<sensor_id>")]
fn get_sensor(db: State<Connection>, sensor_id: String) -> Result<Option<Html<String>>> {
    if let Some((sensor, reading)) = db.get_sensor(&sensor_id)? {
        // let _history = db.select_readings(&sensor_id, &(Local::now() - Duration::minutes(5)))?;
        Ok(Some(Html(templates::SensorTemplate { sensor, reading }.to_string())))
    } else {
        Ok(None)
    }
}

#[get("/sensors/<sensor_id>/json")]
fn get_sensor_json(db: State<Connection>, sensor_id: String) -> Result<Option<Json<Reading>>> {
    Ok(db.get_sensor(&sensor_id)?.map(|(_, reading)| Json(reading)))
}

lazy_static! {
    /// Contains bundled static files.
    static ref STATICS: HashMap<PathBuf, (ContentType, Vec<u8>)> = {
        let mut map = HashMap::new();
        map.insert(
            "favicon-16x16.png".into(),
            (ContentType::PNG, include_bytes!("statics/favicon-16x16.png").to_vec()),
        );
        map.insert(
            "favicon-32x32.png".into(),
            (ContentType::PNG, include_bytes!("statics/favicon-32x32.png").to_vec()),
        );
        map.insert(
            "apple-touch-icon.png".into(),
            (
                ContentType::PNG,
                include_bytes!("statics/apple-touch-icon.png").to_vec(),
            ),
        );
        map.insert(
            "android-chrome-192x192.png".into(),
            (
                ContentType::PNG,
                include_bytes!("statics/android-chrome-192x192.png").to_vec(),
            ),
        );
        map.insert(
            "android-chrome-512x512.png".into(),
            (
                ContentType::PNG,
                include_bytes!("statics/android-chrome-512x512.png").to_vec(),
            ),
        );
        map.insert(
            "bulma.min.css".into(),
            (ContentType::CSS, include_bytes!("statics/bulma.min.css").to_vec()),
        );
        map.insert(
            "bulma-prefers-dark.css".into(),
            (
                ContentType::CSS,
                include_bytes!("statics/bulma-prefers-dark.css").to_vec(),
            ),
        );
        map.insert(
            "chart.js".into(),
            (ContentType::JavaScript, include_bytes!("statics/chart.js").to_vec()),
        );
        map.insert(
            "fontawesome.css".into(),
            (
                ContentType::CSS,
                include_bytes!("statics/fontawesome-free-5.13.1-web/css/all.css").to_vec(),
            ),
        );
        map.insert(
            "fa-solid-900.woff2".into(),
            (
                ContentType::WOFF2,
                include_bytes!("statics/fontawesome-free-5.13.1-web/webfonts/fa-solid-900.woff2").to_vec(),
            ),
        );
        map.insert(
            "fa-regular-400.woff2".into(),
            (
                ContentType::WOFF2,
                include_bytes!("statics/fontawesome-free-5.13.1-web/webfonts/fa-regular-400.woff2").to_vec(),
            ),
        );
        map.insert(
            "fa-brands-400.woff2".into(),
            (
                ContentType::WOFF2,
                include_bytes!("statics/fontawesome-free-5.13.1-web/webfonts/fa-brands-400.woff2").to_vec(),
            ),
        );
        map
    };
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
                dashboard: DashboardSettings::default(),
            },
            Connection::open_and_initialize(":memory:")?,
        )?)?)
    }
}
