//! Application configuration.
//!
//! Manages loading and saving of `config.toml` from `~/.config/AetherCast/`.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application configuration loaded from TOML file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub app: AppConfigSection,
    #[serde(default)]
    pub location: LocationConfig,
    #[serde(default)]
    pub weather: WeatherConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfigSection {
    #[serde(default = "default_app_name")]
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocationConfig {
    #[serde(default = "default_latitude")]
    pub latitude: f64,
    #[serde(default = "default_longitude")]
    pub longitude: f64,
    #[serde(default = "default_city")]
    pub city: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WeatherConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default = "default_temperature_unit")]
    pub temperature_unit: String,
    #[serde(default = "default_wind_unit")]
    pub wind_unit: String,
    #[serde(default = "default_language")]
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiConfig {
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_seconds: u64,
}

// --- Default values ---

fn default_app_name() -> String { "AetherCast".to_string() }
fn default_version() -> String { "0.1.0".to_string() }
fn default_latitude() -> f64 { 52.2297 }
fn default_longitude() -> f64 { 21.0122 }
fn default_city() -> String { "Warsaw".to_string() }
fn default_provider() -> String { "open_meteo".to_string() }
fn default_temperature_unit() -> String { "celsius".to_string() }
fn default_wind_unit() -> String { "kmh".to_string() }
fn default_language() -> String { "en".to_string() }
fn default_log_level() -> String { "info".to_string() }
fn default_refresh_interval() -> u64 { 900 }

/// Returns the default config with sensible values.
pub fn default_config() -> AppConfig {
    AppConfig {
        app: AppConfigSection {
            name: default_app_name(),
            version: default_version(),
            theme: "material".to_string(),
        },
        location: LocationConfig {
            latitude: default_latitude(),
            longitude: default_longitude(),
            city: default_city(),
        },
        weather: WeatherConfig {
            provider: default_provider(),
            temperature_unit: default_temperature_unit(),
            wind_unit: default_wind_unit(),
            language: default_language(),
        },
        logging: LoggingConfig {
            level: default_log_level(),
            file: default_log_file(),
        },
        ui: UiConfig {
            refresh_interval_seconds: default_refresh_interval(),
        },
    }
}

fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("AetherCast")
}

fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

fn default_log_file() -> String {
    if let Some(data_dir) = dirs::data_local_dir() {
        data_dir.join("AetherCast").join("app.log").to_string_lossy().to_string()
    } else {
        "AetherCast.log".to_string()
    }
}

/// Load configuration from the config file, falling back to defaults.
pub fn load_config() -> AppConfig {
    let path = config_path();
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(contents) => {
                match toml::from_str::<AppConfig>(&contents) {
                    Ok(config) => {
                        tracing::info!(config_path = %path.display(), "Configuration loaded from file");
                        config
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, config_path = %path.display(), "Failed to parse config file, using defaults");
                        default_config()
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, config_path = %path.display(), "Failed to read config file, using defaults");
                default_config()
            }
        }
    } else {
        tracing::info!(config_path = %path.display(), "Config file not found, using defaults");
        default_config()
    }
}

/// Save configuration to the config file.
pub fn save_config(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let contents = toml::to_string_pretty(config)?;
    fs::write(&path, contents)?;
    tracing::info!(config_path = %path.display(), "Configuration saved");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = default_config();
        assert_eq!(config.app.name, "AetherCast");
        assert_eq!(config.weather.provider, "open_meteo");
        assert_eq!(config.location.latitude, 52.2297);
    }

    #[test]
    fn test_config_serialization() {
        let config = default_config();
        let toml_str = toml::to_string(&config).expect("Should serialize");
        let parsed: AppConfig = toml::from_str(&toml_str).expect("Should deserialize");
        assert_eq!(parsed.app.name, config.app.name);
        assert_eq!(parsed.weather.provider, config.weather.provider);
    }
}
