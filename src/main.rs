use clap::crate_version;
use log::{debug, info};

mod event;
mod logger;
mod settings;
mod templates;
mod units;
mod web;

/// Entry point.
fn main() {
    logger::init();

    #[rustfmt::skip]
    clap::App::new("My IoT")
        .version(crate_version!())
        .get_matches();

    info!("Reading settings…");
    let settings = settings::read();
    debug!("Settings: {:?}", &settings);

    info!("Starting web server…");
    web::start_server();
}
