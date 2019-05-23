use actix_web::middleware;
use clap::crate_version;
use log::info;

mod event;
mod templates;
mod web;
mod units;

fn main() {
    #[rustfmt::skip]
    clap::App::new("My IoT")
        .version(crate_version!())
        .get_matches();

    std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();
    info!("My IoT is starting…");

    #[rustfmt::skip]
    actix_web::HttpServer::new(|| {
        actix_web::App::new()
            .wrap(middleware::Logger::default())
            .service(web::index)
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .run()
    .unwrap();
}
