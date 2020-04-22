//! Entry point.

use crate::core::supervisor;
use crate::prelude::*;
use dirs::home_dir;
use log::Level;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;

mod consts;
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
    /// Show warning and error messages only
    #[structopt(short = "s", long = "silent", conflicts_with = "verbose")]
    silent: bool,

    /// Show debug messages
    #[structopt(short = "v", long = "verbose", conflicts_with = "silent")]
    verbose: bool,

    /// Settings file
    #[structopt(long, parse(from_os_str), env = "MYIOT_SETTINGS")]
    settings: Option<PathBuf>,

    /// Database URL
    #[structopt(long, env = "MYIOT_DB", default_value = "my-iot.sqlite3")]
    db: String,
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

    info!("Reading settings…");
    let settings = settings::read(
        opt.settings
            .unwrap_or_else(|| home_dir().unwrap().join(".config").join("my-iot.toml")),
    )?;
    debug!("Settings: {:?}", &settings);

    info!("Opening database…");
    let db = Arc::new(Mutex::new(Connection::open_and_initialize(&opt.db)?));

    // Starting up multi-producer multi-consumer bus:
    // - services create and return their input channels
    // - services send their messages out to `dispatcher_tx`
    // - the dispatcher sends out each message from `dispatcher_rx` to the services input channels
    info!("Starting services…");
    let mut bus = Bus::new();
    bus.add_tx()
        .send(Composer::new("my-iot::start").type_(MessageType::ReadNonLogged).into())?;
    core::persistence::thread::spawn(db.clone(), &mut bus)?;
    services::db::spawn(&db, &mut bus)?;
    core::services::spawn_all(&settings, &db, &mut bus)?;
    bus.spawn()?;

    info!("Starting web server on port {}…", settings.http_port);
    web::start_server(settings, db)
}
