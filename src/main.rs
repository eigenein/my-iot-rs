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
use crate::measurement::Measurement;
use crate::services::Service;
use crate::settings::Settings;
use crate::types::ArcMutex;
use log::{debug, info};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::{sync::mpsc::channel, thread};

pub mod db;
pub mod logging;
pub mod measurement;
pub mod receiver;
pub mod services;
pub mod settings;
pub mod statics;
pub mod templates;
pub mod types;
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
    let db = Arc::new(Mutex::new(db::new()));

    info!("Starting services…");
    let (tx, rx) = channel();
    let _service_statuses = start_services(&settings, &db, &tx); // TODO

    info!("Starting measurement receiver…");
    start_measurement_receiver(rx, db.clone());

    info!("Starting web server…");
    web::start_server(settings.http_port.unwrap_or(8081), db.clone())
}

/// Start all configured services.
fn start_services(settings: &Settings, db: &ArcMutex<Db>, tx: &Sender<Measurement>) -> HashMap<String, ()> {
    settings
        .services
        .iter()
        .map(|(service_id, settings)| {
            info!("Starting service `{}`…", service_id);
            debug!("Settings `{}`: {:?}", service_id, settings);
            spawn_service(service_id.clone(), services::new(settings), db.clone(), tx.clone());
            (service_id.clone(), ()) // TODO: return some valid handle.
        })
        .collect()
}

/// Spawn service thread.
///
/// * `service_id`: user-defined service ID.
/// * `service`: service instance.
/// * `db`: main database.
/// * `tx`: measurement sender.
/// * `status`: struct to keep track of the service status.
fn spawn_service(
    service_id: String,
    mut service: Box<dyn Service>,
    db: ArcMutex<Db>,
    tx: Sender<Measurement>,
) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name(service_id.clone())
        .spawn(move || {
            info!("Running service `{}`…", service_id);
            debug!("State `{}`: {:?}", service_id, &service);
            service.run(db, tx);
        })
        .unwrap()
}

/// Start measurement receiver thread.
fn start_measurement_receiver(rx: Receiver<Measurement>, db: ArcMutex<Db>) {
    thread::spawn(move || receiver::run(rx, db));
}
