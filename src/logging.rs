use log::LevelFilter;
use simplelog::{ConfigBuilder, TermLogger, TerminalMode, ThreadLogMode};

use crate::opts::Opts;
use crate::prelude::*;

pub fn init(opts: &Opts) -> Result {
    let mut config_builder = ConfigBuilder::new();
    config_builder
        .set_thread_level(LevelFilter::Off)
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

pub fn log_result<T>(result: &Result<T>, message: fn() -> &'static str) {
    if let Err(error) = result {
        error!("{}: {}", message(), error);
    }
}
