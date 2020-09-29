use log::LevelFilter;
use sentry::integrations::anyhow::capture_anyhow;
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
        .add_filter_allow_str("my_iot");
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

pub trait Log {
    fn log<M: Fn() -> R, R: AsRef<str>>(self, message: M) -> Self;
}

/// Logs the result and submits errors to Sentry.
impl<T> Log for Result<T> {
    fn log<M: Fn() -> R, R: AsRef<str>>(self, message: M) -> Self {
        if let Err(ref error) = self {
            error!("[{}] {}: {}", capture_anyhow(error), message().as_ref(), error);
        }
        self
    }
}
