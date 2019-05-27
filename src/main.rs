use clap::crate_version;
use log::info;

mod event;
mod templates;
mod units;
mod web;

fn main() {
    std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();

    #[rustfmt::skip]
    clap::App::new("My IoT")
        .version(crate_version!())
        .get_matches();

    info!("Starting web server…");
    web::start_server();
}
