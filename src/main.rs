//! Entry point.

#![feature(proc_macro_hygiene, decl_macro)]

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

#[async_std::main]
async fn main() -> Result {
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
        .add_filter_ignore_str("rustls")
        .add_filter_ignore_str("sqlx::query");
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
