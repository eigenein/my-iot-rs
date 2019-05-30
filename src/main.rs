use clap::crate_version;
use log::{debug, info};
use std::{sync::mpsc::channel, thread};

pub mod event;
pub mod logging;
pub mod services;
pub mod settings;
pub mod templates;
pub mod units;
pub mod web;

/// Entry point.
fn main() {
    logging::init();

    #[rustfmt::skip]
    clap::App::new("My IoT")
        .version(crate_version!())
        .get_matches();

    info!("Reading settings…");
    let settings = settings::read();
    debug!("Settings: {:?}", &settings);

    info!("Starting services…");
    let (tx, rx) = channel();
    for service in settings.services {
        debug!("Starting {:?}.", &service);
        let tx = tx.clone();
        thread::spawn(move || {
            services::new(service).run(tx);
        });
    }

    info!("Starting web server…");
    web::start_server();
}
