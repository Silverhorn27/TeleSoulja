use anyhow::{Context, Result};
use log;
use simple_logger::SimpleLogger;
use std::{
    io::{BufRead, Write},
    path::Path,
};

pub fn load_channel_id_or_links<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<String>> {
    let read_file = std::fs::read_to_string(path).with_context(|| "Failed to read input file")?;
    let channels = read_file
        .split('\n')
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();
    Ok(channels)
}

pub fn prompt(message: &str) -> Result<String> {
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(message.as_bytes())?;
    stdout.flush()?;

    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();

    let mut line = String::new();
    stdin.read_line(&mut line)?;
    Ok(line)
}

pub fn init_logger(verbose: u8) -> Result<()> {
    let level = match verbose {
        0 => log::LevelFilter::Off,
        1 => log::LevelFilter::Error,
        2 => log::LevelFilter::Warn,
        3 => log::LevelFilter::Info,
        4 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    SimpleLogger::new()
        .with_level(level)
        .init()
        .with_context(|| "Failed to init logger")
}
