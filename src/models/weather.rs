// Copyright © 2026 AetherCast Contributors
// SPDX-License-Identifier: MIT

//! Weather data models.
//!
//! Defines the core data structures for weather information.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Complete weather data for a single day.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub date: NaiveDate,
    pub temperature: Temperature,
    pub pressure: f32,          // hPa
    pub wind_speed: f32,        // km/h
    pub wind_direction: String, // "N", "NE", "E", …
    pub humidity: u8,           // %
    pub condition: WeatherCondition,
    pub description: String,
    pub icon_code: String,      // maps to Slint icon / SVG
    pub feels_like: f32,
    pub uv_index: Option<f32>,
    pub precipitation: f32,     // mm
    pub visibility: Option<f32>, // km
}

/// Temperature information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Temperature {
    pub current: f32,
    pub min: f32,
    pub max: f32,
    #[serde(default = "default_unit")]
    pub unit: TemperatureUnit,
}

fn default_unit() -> TemperatureUnit {
    TemperatureUnit::Celsius
}

/// Temperature unit.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TemperatureUnit {
    Celsius,
    Fahrenheit,
}

impl std::fmt::Display for TemperatureUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemperatureUnit::Celsius => write!(f, "°C"),
            TemperatureUnit::Fahrenheit => write!(f, "°F"),
        }
    }
}

/// Weather condition enum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WeatherCondition {
    ClearSky,
    MainlyClear,
    PartlyCloudy,
    Overcast,
    Fog,
    Drizzle,
    Rain,
    HeavyRain,
    Snow,
    Thunderstorm,
    Unknown(String),
}

impl std::fmt::Display for WeatherCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeatherCondition::ClearSky => write!(f, "Clear sky"),
            WeatherCondition::MainlyClear => write!(f, "Mainly clear"),
            WeatherCondition::PartlyCloudy => write!(f, "Partly cloudy"),
            WeatherCondition::Overcast => write!(f, "Overcast"),
            WeatherCondition::Fog => write!(f, "Fog"),
            WeatherCondition::Drizzle => write!(f, "Drizzle"),
            WeatherCondition::Rain => write!(f, "Rain"),
            WeatherCondition::HeavyRain => write!(f, "Heavy rain"),
            WeatherCondition::Snow => write!(f, "Snow"),
            WeatherCondition::Thunderstorm => write!(f, "Thunderstorm"),
            WeatherCondition::Unknown(s) => write!(f, "{}", s),
        }
    }
}

/// Convert WMO weather code (Open-Meteo) to our condition.
pub fn wmo_code_to_condition(code: i32) -> WeatherCondition {
    match code {
        0 => WeatherCondition::ClearSky,
        1 | 2 => WeatherCondition::MainlyClear,
        3 => WeatherCondition::Overcast,
        45 | 48 => WeatherCondition::Fog,
        51 | 53 | 55 => WeatherCondition::Drizzle,
        56 | 57 => WeatherCondition::Drizzle,
        61 | 63 | 65 => WeatherCondition::Rain,
        66 | 67 => WeatherCondition::HeavyRain,
        71 | 73 | 75 | 77 => WeatherCondition::Snow,
        80 | 81 | 82 => WeatherCondition::Rain,
        85 | 86 => WeatherCondition::Snow,
        95 | 96 | 99 => WeatherCondition::Thunderstorm,
        _ => WeatherCondition::Unknown(format!("Code {}", code)),
    }
}

/// Get the icon code for a weather condition.
pub fn condition_to_icon(condition: &WeatherCondition) -> &'static str {
    match condition {
        WeatherCondition::ClearSky => "sun",
        WeatherCondition::MainlyClear => "sun-cloud",
        WeatherCondition::PartlyCloudy => "partly-cloudy",
        WeatherCondition::Overcast => "cloud",
        WeatherCondition::Fog => "fog",
        WeatherCondition::Drizzle => "drizzle",
        WeatherCondition::Rain => "rain",
        WeatherCondition::HeavyRain => "heavy-rain",
        WeatherCondition::Snow => "snow",
        WeatherCondition::Thunderstorm => "thunderstorm",
        WeatherCondition::Unknown(_) => "unknown",
    }
}

/// Convert wind direction angle (degrees) to compass label.
pub fn wind_direction_from_angle(degrees: f32) -> String {
    let dirs = ["N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE",
                "S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW"];
    let index = ((degrees + 11.25) / 22.5).floor() as usize % 16;
    dirs[index].to_string()
}

/// Wind direction angle to SVG rotation degrees.
pub fn wind_direction_rotation(degrees: f32) -> f32 {
    degrees
}
