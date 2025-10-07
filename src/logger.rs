
//! Logger configuration.

use std::fs;
use std::fs::OpenOptions;
use std::str::FromStr;
use tracing_subscriber::{Layer, Registry, filter};
use tracing::{debug, info, Level};
use tracing_subscriber::layer::SubscriberExt;

/// Sets the logger global features for the application.
/// * `lvl` - Log level (DEBUG, INFO, WARN, ERROR).
/// * `tf` - Whether to log to file (true/false).
/// * `ts` - Whether to log to stdout (true/false).
/// * `ld` - Directory to store log files (default: "logs").
/// * `file` - Log file name (default: "app.log").
/// # Usage
/// At the start of the application, call this function to set up the logger.
/// ```
/// use templar::logger::set_logger;
/// set_logger(level, to_file, to_stdout, log_dir, log_file).unwrap();
/// ```
/// # Example
/// ```
/// use templar::logger::set_logger;
/// set_logger("INFO", true, true, "logs", "app.log").unwrap();
/// ```
/// # Errors
/// 1) Returns an error if the log directory cannot be created or the log file cannot be opened.
/// 2) Returns an error if the global subscriber cannot be set.
pub fn set_logger(
    lvl:String,
    tf:bool,
    ts:bool,
    ld:String,
    file:String
    ) -> Result<(), Box<dyn std::error::Error>>{

    // Set up the log level and filter.
    let ll = Level::from_str(&lvl).unwrap_or(Level::INFO);
    let lf = filter::LevelFilter::from_level(ll);

    // If stdout logging is enabled, set up the stdout logging layer.
    let lys  = if ts{
        let lys = tracing_subscriber::fmt::layer().compact().with_ansi(true).with_filter(lf.clone());
        Some(lys)
    }else {None};

    // Set up the log file path.
    let p = format!("{d}/{f}",d=ld,f=file);

    // If file logging is enabled, create the directory.
    if tf{fs::create_dir_all(&ld)?;}
    // If file logging is enabled, set up the file logging layer.
    let lyf = if tf{
        let f = OpenOptions::new().append(true).create(true).open(p.clone())?;
        let lyf = tracing_subscriber::fmt::layer().compact().with_ansi(false).with_writer(f).with_filter(lf.clone());
        Some(lyf)
    }else{None};
    const BANNER: &str = r#"
|------------------------------------------|
|   ______                     __          |
|  /_  __/__  ____ ___  ____  / /___ ______|
|   / / / _ \/ __ `__ \/ __ \/ / __ `/ ___/|
|  / / /  __/ / / / / / /_/ / / /_/ / /    |
| /_/  \___/_/ /_/ /_/ .___/_/\__,_/_/     |
|                   /_/                    |
|-----------@isopropilick - 2025-----------|
    "#;
    let s = Registry::default().with(lys).with(lyf);
    tracing::subscriber::set_global_default(s)?;
    info!("{}",BANNER);
    info!("Logger initialized, log level set to: {}",ll);
    if ts{debug!("Logging to stdout.")}
    if tf{debug!("Logging to file: {}", p.replace("\\","\\\\"))}
    Ok(())

}