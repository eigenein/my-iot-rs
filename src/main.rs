use askama::Template;
use clap::{crate_version, App};
use log::info;
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
