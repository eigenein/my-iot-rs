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
//! ## OpenSSL
//!
//! Compiling My IoT may require you to install `pkg-config` and OpenSSL. Most likely, `libssl-dev`
//! is not installed by default on Raspbian.
//!
//! See https://docs.rs/openssl/0.10.24/openssl/#automatic for more information.
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
use clap::Arg;
use failure::Error;
use log::{debug, info};
use multiqueue2::{broadcast_queue, BroadcastReceiver, BroadcastSender};
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

    // This is where all the magic happens.
    // Services send readings to each other with the MPMC queue.
    // TODO: which capacity should I use?
    // TODO: implement back pressure for `try_send`?
    info!("Starting services…");
    let (tx, rx) = broadcast_queue(1024);
    spawn_services(&settings, &db, &tx, &rx)?;

    info!("Starting readings receiver…");
    receiver::start(&rx, db.clone())?;

    // According to the `multiqueue` docs, here we should drop the original sender and receiver.
    drop(tx);
    rx.unsubscribe();

    info!("Starting web server on port {}…", settings.http_port);
    web::start_server(settings, db.clone())
}

/// Spawn all configured services.
fn spawn_services(
    settings: &Settings,
    db: &Arc<Mutex<Db>>,
    tx: &BroadcastSender<Message>,
    rx: &BroadcastReceiver<Message>,
) -> Result<()> {
    for (service_id, settings) in settings.services.iter() {
        info!("Spawning service `{}`…", service_id);
        debug!("Settings `{}`: {:?}", service_id, settings);
        services::new(service_id, settings)?.spawn(db.clone(), &tx, &rx)?;
    }
    Ok(())
}
