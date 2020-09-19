//! Entry point.

#![feature(proc_macro_hygiene, decl_macro)]

use structopt::StructOpt;

use crate::prelude::*;

mod core;
mod format;
mod logging;
mod opts;
mod prelude;
mod services;
mod settings;
mod web;

#[async_std::main]
async fn main() -> Result {
    let opts = opts::Opts::from_args();
    if opts.version {
        // I want to print only the version, without the application name.
        println!("{}", crate_version!());
        return Ok(());
    }

    logging::init(&opts)?;

    info!("Reading the settings…");
    let settings = settings::read(opts.settings)?;
    debug!("Settings: {:?}", &settings);

    info!("Opening the database…");
    let db = Connection::open(&opts.db).await?;

    info!("Starting services…");
    let mut bus = Bus::new();
    core::db::tasks::spawn(db.clone(), &mut bus);
    services::db::Db.spawn("system::db".into(), &mut bus, db.clone());
    services::spawn_all(&settings, &opts.service_ids, &mut bus, &db).await?;

    if !opts.no_web_server {
        std::thread::spawn(move || web::start_server(&settings, db));
    } else {
        warn!("Web server is disabled.");
    }

    bus.spawn().await
}
