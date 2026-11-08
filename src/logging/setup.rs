// Copyright © 2026 AetherCast Contributors
// SPDX-License-Identifier: MIT

//! Logging initialization.
//!
//! Sets up tracing-subscriber with both console and file output.
//! Log file location is configurable via config.

use std::fs;
use std::path::PathBuf;
use tracing_subscriber::{
    EnvFilter,
    util::SubscriberInitExt,
};

/// Initialize the global tracing subscriber.
///
/// # Arguments
/// * `log_dir` — Directory for log files
/// * `log_level` — Log level filter (e.g. "info", "debug")
pub fn init_logging(log_dir: &str, log_level: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create log directory if it doesn't exist
    let log_dir_path = PathBuf::from(log_dir);
    fs::create_dir_all(&log_dir_path)?;

    let log_file_path = log_dir_path.join("app.log");

    // Filter layer — controls log levels
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    // Use a simple console layer
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_writer(std::io::stdout)
        .init();

    tracing::info!(
        log_file = %log_file_path.display(),
        log_level = log_level,
        "Logging initialized"
    );

    Ok(())
}

/// Get the default log directory path.
pub fn default_log_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join("AetherCast")
}
