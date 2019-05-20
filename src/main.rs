extern crate askama;
#[macro_use]
extern crate clap;

use askama::Template;
use clap::App;
use warp::{self, path, Filter};

#[derive(Template)]
#[template(path = "index.html")]
struct Index;


fn main() {
    App::new("My IoT")
        .version(crate_version!())
        .get_matches();

    warp::serve(
        path::end().map(|| warp::reply::html(Index{}.render().unwrap()))
    ).run(([127, 0, 0, 1], 8080));
}
