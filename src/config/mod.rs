//! Configuration module.
//!
//! Loads and saves application settings from a TOML config file.

pub mod app_config;

pub use app_config::{AppConfig, default_config, load_config, save_config};
