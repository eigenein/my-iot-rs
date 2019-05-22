extern crate askama;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;

use askama::Template;
use clap::App;
use warp::{self, path, Filter};

#[derive(Template)]
#[template(path = "index.html")]
struct Index;

fn main() {
    #[rustfmt::skip]
    App::new("My IoT")
        .version(crate_version!())
        .get_matches();

    pretty_env_logger::init();
    info!("My IoT is starting…");

    #[rustfmt::skip]
    warp::serve(
        path::end().map(|| warp::reply::html(Index{}.render().unwrap()))
    ).run(([127, 0, 0, 1], 8080));
}
