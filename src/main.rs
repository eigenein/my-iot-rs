//! Entry point.

use crate::db::Db;
use crate::prelude::*;
use crate::settings::Settings;
use crossbeam_channel::{Receiver, Sender};
use log::Level;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;

mod consts;
mod core;
mod db;
mod format;
mod persistence;
mod prelude;
mod services;
mod settings;
mod supervisor;
mod templates;
mod web;

#[derive(StructOpt, Debug)]
#[structopt(name = "my-iot", author, about)]
struct Opt {
    /// Show warning and error messages only
    #[structopt(short = "s", long = "silent")]
    silent: bool,

    /// Show debug messages
    #[structopt(short = "v", long = "verbose", conflicts_with = "silent")]
    verbose: bool,

    /// Settings file
    #[structopt(long, parse(from_os_str), env = "MYIOT_SETTINGS", default_value = "my-iot.toml")]
    settings: PathBuf,

    /// Database file
    #[structopt(long, parse(from_os_str), env = "MYIOT_DB", default_value = "my-iot.sqlite3")]
    db: PathBuf,
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
    let settings = settings::read(opt.settings)?;
    debug!("Settings: {:?}", &settings);

    info!("Opening database…");
    let db = Arc::new(Mutex::new(Db::new(opt.db)?));

    // Starting up multi-producer multi-consumer bus:
    // - services create and return their input channels
    // - services send their messages out to `dispatcher_tx`
    // - the dispatcher sends out each message from `dispatcher_rx` to the services input channels
    info!("Starting services…");
    let (dispatcher_tx, dispatcher_rx) = crossbeam_channel::unbounded();
    dispatcher_tx.send(Composer::new("my-iot::start").type_(MessageType::ReadNonLogged).into())?;
    let mut all_txs = vec![persistence::spawn(db.clone(), &dispatcher_tx)?];
    all_txs.extend(spawn_services(&settings, &db, &dispatcher_tx)?);
    spawn_dispatcher(dispatcher_rx, dispatcher_tx, all_txs)?;

    info!("Starting web server on port {}…", settings.http_port);
    web::start_server(settings, db.clone())
}

/// Spawn all configured services.
///
/// Returns a vector of all service input message channels.
///
/// - `tx`: output message channel that's used by services to send their messages to.
fn spawn_services(settings: &Settings, db: &Arc<Mutex<Db>>, tx: &Sender<Message>) -> Result<Vec<Sender<Message>>> {
    let mut service_txs = Vec::new();

    for (service_id, service_settings) in settings.services.iter() {
        if !settings.disabled_services.contains(service_id.as_str()) {
            info!("Spawning service `{}`…", service_id);
            debug!("Settings `{}`: {:?}", service_id, service_settings);
            service_txs.extend(services::spawn(service_id, service_settings, &db, tx)?);
        } else {
            warn!("Service `{}` is disabled.", &service_id);
        }
    }

    Ok(service_txs)
}

/// Spawn message dispatcher that broadcasts every received message to emulate
/// a multi-producer multi-consumer queue.
///
/// Thus, services exchange messages with each other. Each message from the input channel is
/// broadcasted to each of output channels.
///
/// - `rx`: dispatcher input message channel
/// - `tx`: dispatcher output message channel
/// - `txs`: service output message channels
fn spawn_dispatcher(rx: Receiver<Message>, tx: Sender<Message>, txs: Vec<Sender<Message>>) -> Result<()> {
    info!("Spawning message dispatcher…");
    supervisor::spawn("my-iot::dispatcher", tx, move || -> Result<()> {
        for message in &rx {
            debug!("Dispatching {}", &message.sensor);
            for tx in txs.iter() {
                tx.send(message.clone())?;
            }
            debug!("Dispatched {}", &message.sensor);
        }
        Err(format_err!("Receiver channel is unexpectedly exhausted"))
    })?;
    Ok(())
}
