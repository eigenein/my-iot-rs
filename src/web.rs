//! Implements web server.

use crate::prelude::*;
use crate::settings::Settings;
use crate::templates;
use lazy_static::lazy_static;
use rocket::config::Environment;
use rocket::http::ContentType;
use rocket::response::content::{Content, Html};
use rocket::{get, routes, Config, Rocket, State};
use rocket_contrib::json::Json;
use std::collections::HashMap;

const FAVICON: &[u8] = include_bytes!("statics/favicon.ico");

struct MaxSensorAgeMs(i64);

/// Start the web application.
pub fn start_server(settings: &Settings, db: Arc<Mutex<Connection>>) -> Result<()> {
    Err(Box::new(make_rocket(settings, db)?.launch()))
}

fn make_rocket(settings: &Settings, db: Arc<Mutex<Connection>>) -> Result<Rocket> {
    Ok(rocket::custom(
        Config::build(Environment::Production)
            .port(settings.http_port)
            .keep_alive(60)
            .finalize()?,
    )
    .manage(MaxSensorAgeMs(settings.max_sensor_age_ms))
    .manage(db)
    .manage(settings.clone())
    .mount(
        "/",
        routes![
            get_index,
            get_sensors,
            get_settings,
            get_favicon,
            get_static,
            get_sensor,
            get_sensor_json
        ],
    ))
}

#[get("/")]
fn get_index() -> Html<String> {
    Html(templates::IndexTemplate::new().to_string())
}

#[get("/sensors")]
fn get_sensors(db: State<Arc<Mutex<Connection>>>, max_sensor_age_ms: State<MaxSensorAgeMs>) -> Result<Html<String>> {
    Ok(Html(
        templates::SensorsTemplate::new(&db.lock().unwrap(), max_sensor_age_ms.0)?.to_string(),
    ))
}

#[get("/settings")]
fn get_settings(settings: State<Settings>) -> Result<Html<String>> {
    Ok(Html(templates::SettingsTemplate::new(&settings)?.to_string()))
}

#[get("/favicon.ico")]
fn get_favicon() -> Content<&'static [u8]> {
    Content(ContentType::Icon, FAVICON)
}

#[get("/static/<key>")]
fn get_static(key: String) -> Option<Content<&'static [u8]>> {
    STATICS
        .get(key.as_str())
        .map(|(content_type, content)| Content(content_type.clone(), &content[..]))
}

#[get("/sensors/<sensor_id>")]
fn get_sensor(db: State<Arc<Mutex<Connection>>>, sensor_id: String) -> Result<Option<Html<String>>> {
    Ok(db
        .lock()
        .unwrap()
        .get_sensor(&sensor_id)?
        .map(|(sensor, reading)| Html(templates::SensorTemplate::new(sensor, reading).to_string())))
}

#[get("/sensors/<sensor_id>/json")]
fn get_sensor_json(db: State<Arc<Mutex<Connection>>>, sensor_id: String) -> Result<Option<Json<Reading>>> {
    Ok(db
        .lock()
        .unwrap()
        .get_sensor(&sensor_id)?
        .map(|(_, reading)| Json(reading)))
}

lazy_static! {
    static ref STATICS: HashMap<&'static str, (ContentType, Vec<u8>)> = {
        let mut map = HashMap::new();
        map.insert(
            "favicon-16x16.png",
            (ContentType::PNG, include_bytes!("statics/favicon-16x16.png").to_vec()),
        );
        map.insert(
            "favicon-32x32.png",
            (ContentType::PNG, include_bytes!("statics/favicon-32x32.png").to_vec()),
        );
        map.insert(
            "apple-touch-icon.png",
            (
                ContentType::PNG,
                include_bytes!("statics/apple-touch-icon.png").to_vec(),
            ),
        );
        map.insert(
            "android-chrome-192x192.png",
            (
                ContentType::PNG,
                include_bytes!("statics/android-chrome-192x192.png").to_vec(),
            ),
        );
        map.insert(
            "android-chrome-512x512.png",
            (
                ContentType::PNG,
                include_bytes!("statics/android-chrome-512x512.png").to_vec(),
            ),
        );
        map.insert(
            "bulma.min.css",
            (ContentType::CSS, include_bytes!("statics/bulma.min.css").to_vec()),
        );
        map.insert(
            "bulma-prefers-dark.css",
            (
                ContentType::CSS,
                include_bytes!("statics/bulma-prefers-dark.css").to_vec(),
            ),
        );
        map.insert(
            "plotly-1.5.0.min.js",
            (
                ContentType::JavaScript,
                include_bytes!("statics/plotly-1.5.0.min.js").to_vec(),
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
                max_sensor_age_ms: default_max_sensor_age_ms(),
                services: HashMap::new(),
            },
            Arc::new(Mutex::new(Connection::open_and_initialize(":memory:")?)),
        )?)?)
    }
}
