//! # Getting started
//!
//! Grab a release from [GitHub](https://github.com/eigenein/my-iot-rs/releases) for your architecture
//! or optionally clone the repo and build it manually:
//!
//! ```sh
//! git clone https://github.com/eigenein/my-iot-rs.git
//! cd my-iot-rs
//! make
//! ```
//!
//! The above should produce a single executable, you can install it to `/usr/local/bin` by running:
//!
//! ```sh
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
use crate::reading::Reading;
use crate::settings::Settings;
use crate::threading::ArcMutex;
use log::{debug, info};
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

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

/// Entry point.
fn main() -> ! {
    logging::init();

    #[rustfmt::skip]
    clap::App::new("My IoT")
        .version(clap::crate_version!())
        .author(clap::crate_authors!("\n"))
        .about(clap::crate_description!())
        .get_matches();

    info!("Reading settings…");
    let settings = settings::read(); // TODO: CLI parameter.
    debug!("Settings: {:?}", &settings);

    info!("Opening database…");
    let db = Arc::new(Mutex::new(Db::new("my-iot.sqlite3")));

    info!("Starting services…");
    let (tx, rx) = channel();
    spawn_services(&settings, &db, &tx);

    info!("Starting readings receiver…");
    receiver::start(rx, db.clone());

    info!("Starting web server…");
    web::start_server(settings, db.clone())
}

/// Spawn all configured services.
fn spawn_services(settings: &Settings, db: &ArcMutex<Db>, tx: &Sender<Reading>) -> Vec<JoinHandle<()>> {
    settings
        .services
        .iter()
        .flat_map(|(service_id, settings)| {
            info!("Spawning service `{}`…", service_id);
            debug!("Settings `{}`: {:?}", service_id, settings);
            let handles = services::new(settings).spawn(service_id.clone(), db.clone(), tx.clone());
            for handle in handles.iter() {
                info!("Spawned thread `{}`.", handle.thread().name().unwrap_or("anonymous"));
            }
            handles
        })
        .collect()
}
