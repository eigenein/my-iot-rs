//! Entry point.

use crate::db::Db;
use crate::message::Message;
use crate::settings::Settings;
use clap::Arg;
use crossbeam_channel::{Receiver, Sender};
use failure::Error;
use log::{debug, info, warn};
use std::sync::{Arc, Mutex};

pub mod consts;
pub mod db;
pub mod logging;
pub mod message;
pub mod persistence;
pub mod services;
pub mod settings;
pub mod templates;
pub mod threading;
pub mod value;
pub mod web;

type Result<T> = std::result::Result<T, Error>;

const DEFAULT_SETTINGS_PATH: &str = "my-iot.toml";
const DEFAULT_DB_PATH: &str = "my-iot.sqlite3";

/// Entry point.
fn main() -> Result<()> {
    logging::init();

    let matches = clap::App::new("My IoT")
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .arg(
            Arg::with_name("settings")
                .short("s")
                .long("settings")
                .takes_value(true)
                .help(&format!("Settings file path (default: {})", DEFAULT_SETTINGS_PATH)),
        )
        .arg(
            Arg::with_name("db")
                .long("--db")
                .takes_value(true)
                .help(&format!("Database file path (default: {})", DEFAULT_DB_PATH)),
        )
        .get_matches();

    info!("Reading settings…");
    let settings = settings::read(matches.value_of("settings").unwrap_or(DEFAULT_SETTINGS_PATH))?;
    debug!("Settings: {:?}", &settings);

    info!("Opening database…");
    let db = Arc::new(Mutex::new(Db::new(matches.value_of("db").unwrap_or(DEFAULT_DB_PATH))?));

    info!("Starting services…");
    // Starting up multi-producer multi-consumer bus.
    let (dispatcher_tx, dispatcher_rx) = crossbeam_channel::unbounded();
    let mut all_txs = vec![persistence::spawn(db.clone(), &dispatcher_tx)?];
    all_txs.extend(spawn_services(&settings, &db, &dispatcher_tx)?);
    spawn_dispatcher(dispatcher_rx, all_txs)?;

    info!("Starting web server on port {}…", settings.http_port);
    web::start_server(settings, db.clone())
}

/// Spawn all configured services.
fn spawn_services(settings: &Settings, db: &Arc<Mutex<Db>>, tx: &Sender<Message>) -> Result<Vec<Sender<Message>>> {
    let mut service_txs = Vec::new();

    for (service_id, service_settings) in settings.services.iter() {
        if !settings.disabled_services.contains(service_id.as_str()) {
            info!("Spawning service `{}`…", service_id);
            debug!("Settings `{}`: {:?}", service_id, service_settings);
            service_txs.extend(services::spawn(service_id, service_settings, &db, tx)?);
        } else {
            warn!("Service `{}` is disabled.", &service_id);
        }
    }

    Ok(service_txs)
}

/// Spawn message dispatcher that broadcasts every received message to emulate
/// multi-producer multi-consumer queue.
/// Thus, services exchange messages with each other.
fn spawn_dispatcher(rx: Receiver<Message>, txs: Vec<Sender<Message>>) -> Result<()> {
    info!("Spawning message dispatcher…");
    threading::spawn("my-iot::dispatcher", move || {
        for message in rx {
            debug!("Dispatching {}", &message.reading.sensor);
            for tx in txs.iter() {
                tx.send(message.clone()).unwrap();
            }
            debug!("Dispatched {}", &message.reading.sensor);
        }
    })?;
    Ok(())
}
