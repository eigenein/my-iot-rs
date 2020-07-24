//! Entry point.

#![feature(proc_macro_hygiene, decl_macro)]

use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use log::LevelFilter;
use simplelog::{ConfigBuilder, TermLogger, TerminalMode, ThreadLogMode};
use structopt::StructOpt;

use crate::opts::Opts;
use crate::prelude::*;

mod core;
mod format;
mod opts;
mod prelude;
mod services;
mod settings;
mod web;

/// Entry point.
fn main() -> Result {
    let opts = opts::Opts::from_args();
    if opts.version {
        // I want to print only the version, without the application name.
        println!("{}", crate_version!());
        return Ok(());
    }

    init_logging(&opts)?;

    info!("Reading the settings…");
    let settings = settings::read(opts.settings)?;
    debug!("Settings: {:?}", &settings);

    info!("Opening the database…");
    let db = Connection::open_and_initialize(&opts.db)?;

    info!("Starting services…");
    let message_counter = Arc::new(AtomicU64::new(0));
    let mut bus = Bus::new(message_counter.clone());
    bus.add_tx()
        .send(Message::new("my-iot::start").type_(MessageType::ReadNonLogged))?;
    core::db::thread::spawn(db.clone(), &mut bus)?;
    services::db::Db.spawn("system::db".into(), &mut bus, db.clone())?;
    services::spawn_all(&settings, &opts.service_ids, &mut bus, &db)?;
    bus.spawn()?;

    info!("Starting web server on port {}…", settings.http.port);
    web::start_server(&settings, db, message_counter)
}

fn init_logging(opts: &Opts) -> Result {
    let mut config_builder = ConfigBuilder::new();
    config_builder
        .set_thread_level(LevelFilter::Error)
        .set_target_level(LevelFilter::Error)
        .set_location_level(LevelFilter::Debug)
        .set_thread_mode(ThreadLogMode::Names)
        .set_time_format_str("%F %T%.3f")
        .set_time_to_local(true)
        .add_filter_ignore_str("h2")
        .add_filter_ignore_str("hyper")
        .add_filter_ignore_str("launch_")
        .add_filter_ignore_str("reqwest")
        .add_filter_ignore_str("rustls");
    if opts.suppress_log_timestamps {
        config_builder.set_time_level(LevelFilter::Off);
    }
    TermLogger::init(
        if opts.silent {
            LevelFilter::Warn
        } else if opts.verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        },
        config_builder.build(),
        TerminalMode::Stderr,
    )?;
    Ok(())
}
