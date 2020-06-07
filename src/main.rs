//! Entry point.

#![feature(proc_macro_hygiene, decl_macro)]

use crate::prelude::*;
use log::LevelFilter;
use simplelog::{ConfigBuilder, TermLogger, TerminalMode, ThreadLogMode};
use std::path::PathBuf;
use structopt::StructOpt;

mod core;
mod format;
mod prelude;
mod services;
mod settings;
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
    init_logging(opt.silent, opt.verbose)?;

    info!("Reading the settings…");
    let settings = settings::read(opt.settings)?;
    debug!("Settings: {:?}", &settings);

    info!("Opening the database…");
    let db = Connection::open_and_initialize(&opt.db)?;

    info!("Starting services…");
    let mut bus = Bus::new();
    bus.add_tx()
        .send(Message::new("my-iot::start").type_(MessageType::ReadNonLogged))?;
    core::persistence::thread::spawn(db.clone(), &mut bus)?;
    services::db::Db.spawn("system::db".into(), &mut bus, db.clone())?;
    core::services::spawn_all(&settings, &mut bus)?;
    bus.spawn()?;

    info!("Starting web server on port {}…", settings.http_port);
    web::start_server(&settings, db)
}

fn init_logging(silent: bool, verbose: bool) -> Result<()> {
    TermLogger::init(
        if silent {
            LevelFilter::Warn
        } else if verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        },
        ConfigBuilder::new()
            .set_thread_level(LevelFilter::Error)
            .set_target_level(LevelFilter::Error)
            .set_location_level(LevelFilter::Debug)
            .set_thread_mode(ThreadLogMode::Names)
            .set_time_format_str("%F %T%.3f")
            .set_time_to_local(true)
            .add_filter_ignore_str("launch_")
            .build(),
        TerminalMode::Stderr,
    )?;
    Ok(())
}
