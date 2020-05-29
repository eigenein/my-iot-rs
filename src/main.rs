//! Entry point.

#![feature(proc_macro_hygiene, decl_macro)]

use crate::core::supervisor;
use crate::prelude::*;
use log::Level;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;

mod core;
mod errors;
mod format;
mod prelude;
mod services;
mod settings;
mod templates;
mod web;

#[derive(StructOpt, Debug)]
#[structopt(name = "my-iot", author, about)]
struct Opt {
    /// Show only warnings and errors
    #[structopt(short = "s", long = "silent", conflicts_with = "verbose")]
    silent: bool,

    /// Show all log messages
    #[structopt(short = "v", long = "verbose", conflicts_with = "silent")]
    verbose: bool,

    /// Database URL
    #[structopt(long, env = "MYIOT_DB", default_value = "my-iot.sqlite3")]
    db: String,

    /// Settings files
    #[structopt(parse(from_os_str), env = "MYIOT_SETTINGS", default_value = "my-iot.toml")]
    settings: Vec<PathBuf>,
}

/// Entry point.
fn main() -> Result<()> {
    let opt: Opt = Opt::from_args();
    simple_logger::init_with_level(if opt.silent {
        Level::Warn
    } else if opt.verbose {
        Level::Debug
    } else {
        Level::Info
    })?;

    info!("Reading the settings…");
    let settings = settings::read(opt.settings)?;
    debug!("Settings: {:?}", &settings);

    info!("Opening the database…");
    let db = Arc::new(Mutex::new(Connection::open_and_initialize(&opt.db)?));

    crossbeam::thread::scope(|scope| -> Result<()> {
        info!("Starting services…");
        let mut bus = Bus::new();
        bus.add_tx()
            .send(Message::new("my-iot::start").type_(MessageType::ReadNonLogged))?;
        core::persistence::thread::spawn(scope, db.clone(), &mut bus)?;
        services::db::Db.spawn(scope, "system::db", &mut bus, db.clone())?;
        core::services::spawn_all(scope, &settings, &mut bus)?;
        bus.spawn(scope)?;

        info!("Starting web server on port {}…", settings.http_port);
        web::start_server(&settings, db)
    })
    .unwrap()?;

    Ok(())
}
