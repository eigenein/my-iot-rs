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

use log::{debug, info};
use std::sync::{Arc, Mutex};
use std::{sync::mpsc::channel, thread};

pub mod db;
pub mod logging;
pub mod measurement;
pub mod receiver;
pub mod services;
pub mod settings;
pub mod templates;
pub mod value;
pub mod web;

/// Entry point.
fn main() {
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
    for service in settings.services {
        let db = db.clone();
        let tx = tx.clone();
        thread::spawn(move || {
            debug!("Starting {:?}…", &service);
            let mut service = services::new(service);
            debug!("Running {:?}…", &service);
            service.run(db, tx);
        });
    }

    info!("Starting measurement receiver…");
    {
        let db = db.clone();
        thread::spawn(move || {
            receiver::run(rx, db);
        });
    }

    info!("Starting web server…");
    web::start_server(settings.http_port.unwrap_or(8081), db.clone());
}
