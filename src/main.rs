//! # Getting started
//!
//! Grab a release from [GitHub](https://github.com/eigenein/my-iot-rs/releases) for your architecture
//!
//! or, install it via `cargo`:
//!
//! ```sh
//! cargo install my-iot
//! ```
//!
//! or, clone the repo and build it manually:
//!
//! ```sh
//! git clone https://github.com/eigenein/my-iot-rs.git
//! cd my-iot-rs
//! make
//! sudo make install
//! ```
//!
//! Then, you'll need to create a configuration file `settings.yml`. It must contain exactly one object,
//! please read the [`settings`](settings/index.html) documentation.
//!
//! ## File capabilities
//!
//! If you're not using `make`, you may need to manually set capabilities on the produced binary:
//!
//! ```sh
//! setcap cap_net_raw+ep /usr/local/bin/my-iot
//! ```
//!
//! This is needed to use some low-level protocols (for instance, ICMP) as a non-root user.

use crate::db::Db;
use crate::reading::Message;
use crate::settings::Settings;
use bus::Bus;
use clap::Arg;
use crossbeam_channel::{Receiver, Sender};
use failure::Error;
use log::{debug, info};
use std::sync::{Arc, Mutex};

pub mod consts;
pub mod db;
pub mod logging;
pub mod reading;
pub mod receiver;
pub mod services;
pub mod settings;
pub mod templates;
pub mod threading;
pub mod value;
pub mod web;

type Result<T> = std::result::Result<T, Error>;

const DEFAULT_SETTINGS_PATH: &str = "settings.yml";
const DEFAULT_DB_PATH: &str = "my-iot.sqlite3";

/// Entry point.
fn main() -> Result<()> {
    logging::init();

    let matches = clap::App::new("My IoT")
        .version(clap::crate_version!())
        .author(clap::crate_authors!("\n"))
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
    let (tx, rx) = crossbeam_channel::bounded(1024);
    let mut bus = Bus::new(1024);
    receiver::spawn(&mut bus, db.clone())?;
    spawn_services(&settings, &db, &tx, &mut bus)?;
    spawn_dispatcher(rx, bus)?;

    info!("Starting web server on port {}…", settings.http_port);
    web::start_server(settings, db.clone())
}

/// Spawn all configured services.
fn spawn_services(
    settings: &Settings,
    db: &Arc<Mutex<Db>>,
    tx: &Sender<Message>,
    bus: &mut Bus<Message>,
) -> Result<()> {
    for (service_id, settings) in settings.services.iter() {
        info!("Spawning service `{}`…", service_id);
        debug!("Settings `{}`: {:?}", service_id, settings);
        services::new(service_id, settings)?.spawn(db.clone(), &tx, bus)?;
    }
    Ok(())
}

/// Spawn message dispatcher that broadcasts every received message to emulate
/// multi-producer multi-consumer queue.
/// Thus, services exchange messages with each other.
fn spawn_dispatcher(rx: Receiver<Message>, mut bus: Bus<Message>) -> Result<()> {
    info!("Spawning message dispatcher…");
    threading::spawn("my-iot::dispatcher", move || loop {
        bus.broadcast(rx.recv().unwrap());
    })?;
    Ok(())
}
