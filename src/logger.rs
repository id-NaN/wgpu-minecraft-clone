use chrono::Local;
use color_eyre::Result;
use colored::{Color, Colorize};
use log::{set_logger, set_max_level, Level, Log, Metadata, Record};

use crate::SETTINGS;

static LOGGER: Logger = Logger {};

fn get_color(level: Level) -> Color {
    match level {
        Level::Error => Color::Red,
        Level::Warn => Color::Yellow,
        Level::Info => Color::White,
        Level::Debug => Color::Green,
        Level::Trace => Color::Cyan,
    }
}

pub fn initialize_logger() -> Result<()> {
    set_logger(&LOGGER)?;
    // set log level so the settings get loaded with functional logger.
    set_max_level(SETTINGS.log.max_level);
    Ok(())
}

pub struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        println!(
            "[{}] [{:40}] [{:5}] {}",
            Local::now().format("%FT%T"),
            format!(
                "{}:{}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
            )
            .magenta(),
            record.level().as_str().blue(),
            format!("{}", record.args()).color(get_color(record.level())),
        )
    }

    fn flush(&self) {}
}
