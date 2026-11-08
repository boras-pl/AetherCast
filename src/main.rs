// Copyright © 2026 AetherCast Contributors
// SPDX-License-Identifier: MIT

//! AetherCast — Main Entry Point
//!
//! A desktop weather application with Slint UI and multi-provider weather APIs.

mod config;
mod logging;
mod models;
mod services;

use config::{AppConfig, load_config, save_config};
use logging::init_logging;
use models::weather;
use services::*;

slint::include_modules!();

/// Convert a Rust WeatherData to a Slint WeatherData struct.
fn to_slint_weather(data: &weather::WeatherData) -> WeatherData {
    WeatherData {
        date: data.date.to_string().into(),
        temperature: format!("{}°C", data.temperature.current).into(),
        temp_min: format!("{}°C", data.temperature.min).into(),
        temp_max: format!("{}°C", data.temperature.max).into(),
        feels_like: format!("{}°C", data.feels_like).into(),
        pressure: format!("{:.0}", data.pressure).into(),
        wind_speed: format!("{:.0} km/h", data.wind_speed).into(),
        wind_direction: data.wind_direction.clone().into(),
        humidity: format!("{}%", data.humidity).into(),
        condition: data.condition.to_string().into(),
        description: data.description.clone().into(),
        icon_code: data.icon_code.clone().into(),
        precipitation: format!("{:.1}", data.precipitation).into(),
        uv_index: if let Some(uv) = data.uv_index {
            format!("{:.1}", uv).into()
        } else {
            "".into()
        },
        visibility: if let Some(vis) = data.visibility {
            format!("{:.1}", vis).into()
        } else {
            "".into()
        },
    }
}

/// Convert a Vec of WeatherData to a Slint array model.
fn to_slint_week(week: &[weather::WeatherData]) -> slint::ModelRc<WeatherData> {
    let vec: Vec<WeatherData> = week.iter().map(to_slint_weather).collect();
    slint::ModelRc::from(std::rc::Rc::new(slint::VecModel::from(vec)))
}

/// Creates all available providers and tries them in order.
fn fetch_weather_sync(config: &AppConfig) -> Result<(weather::WeatherData, weather::WeatherData, Vec<weather::WeatherData>), String> {
    let lat = config.location.latitude;
    let lon = config.location.longitude;

    // Build provider list based on config
    let mut providers: Vec<Box<dyn WeatherService>> = Vec::new();

    // Primary provider
    match config.weather.provider.as_str() {
        "open_meteo" => {
            providers.push(Box::new(OpenMeteoProvider::new()));
        }
        "wttr_in" => {
            providers.push(Box::new(WttrInProvider::new()));
        }
        "open_weather_map" => {
            if let Some(provider) = OpenWeatherMapProvider::from_env() {
                providers.push(Box::new(provider));
            } else {
                tracing::warn!("OpenWeatherMap API key not set (env: OPENWEATHERMAP_API_KEY)");
            }
        }
        "weather_api" => {
            if let Some(provider) = WeatherApiProvider::from_env() {
                providers.push(Box::new(provider));
            } else {
                tracing::warn!("WeatherAPI key not set (env: WEATHERAPI_KEY)");
            }
        }
        "visual_crossing" => {
            if let Some(provider) = VisualCrossingProvider::from_env() {
                providers.push(Box::new(provider));
            } else {
                tracing::warn!("Visual Crossing API key not set (env: VISUALCROSSING_API_KEY)");
            }
        }
        _ => {
            tracing::warn!("Unknown provider '{}', using Open-Meteo", config.weather.provider);
            providers.push(Box::new(OpenMeteoProvider::new()));
        }
    }

    // Add secondary providers as fallbacks
    if !providers.iter().any(|p| p.name() == "Open-Meteo") {
        providers.push(Box::new(OpenMeteoProvider::new()));
    }
    if !providers.iter().any(|p| p.name() == "wttr.in") {
        providers.push(Box::new(WttrInProvider::new()));
    }
    if let Some(provider) = OpenWeatherMapProvider::from_env() {
        if !providers.iter().any(|p| p.name() == "OpenWeatherMap") {
            providers.push(Box::new(provider));
        }
    }
    if let Some(provider) = WeatherApiProvider::from_env() {
        if !providers.iter().any(|p| p.name() == "WeatherAPI.com") {
            providers.push(Box::new(provider));
        }
    }
    if let Some(provider) = VisualCrossingProvider::from_env() {
        if !providers.iter().any(|p| p.name() == "Visual Crossing") {
            providers.push(Box::new(provider));
        }
    }

    // Always add mock as last fallback
    providers.push(Box::new(MockProvider::new()));

    // Try each provider in order
    let mut last_error = None;
    for provider in &providers {
        tracing::info!(provider = provider.name(), "Trying weather provider");
        match tokio::runtime::Runtime::new().unwrap().block_on(provider.fetch(lat, lon)) {
            Ok((today, tomorrow, week)) => {
                tracing::info!(provider = provider.name(), "Successfully fetched weather data");
                return Ok((today, tomorrow, week));
            }
            Err(e) => {
                tracing::warn!(provider = provider.name(), error = %e, "Provider failed, trying next");
                last_error = Some(e);
            }
        }
    }

    Err(format!(
        "All providers failed. Last error: {:?}",
        last_error
    ))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config
    let config = load_config();

    // Initialize logging
    let log_dir = config.logging.file
        .strip_suffix(".log")
        .unwrap_or(&config.logging.file);
    init_logging(log_dir, &config.logging.level)?;

    tracing::info!(
        app = config.app.name,
        version = config.app.version,
        city = config.location.city,
        latitude = config.location.latitude,
        longitude = config.location.longitude,
        provider = config.weather.provider,
        "Starting AetherCast"
    );

    // Create the Slint UI
    let app = AppWindow::new()?;

    // Set initial loading state
    app.set_loading(true);

    // Set up the refresh callback
    {
        let app_clone = app.clone_strong();
        app.on_refresh_weather(move || {
            let app = app_clone.clone_strong();
            // Run fetch on the event loop thread
            let config = load_config();
            match fetch_weather_sync(&config) {
                Ok((today, tomorrow, week)) => {
                    app.set_today_weather(to_slint_weather(&today));
                    app.set_tomorrow_weather(to_slint_weather(&tomorrow));
                    app.set_week_weather(to_slint_week(&week));
                    app.set_loading(false);
                    app.set_error_message("".into());
                }
                Err(e) => {
                    app.set_loading(false);
                    app.set_error_message(format!("Failed to load weather data: {}", e).into());
                }
            }
        });
    }

    // Run initial fetch synchronously
    let config = load_config();
    match fetch_weather_sync(&config) {
        Ok((today, tomorrow, week)) => {
            app.set_today_weather(to_slint_weather(&today));
            app.set_tomorrow_weather(to_slint_weather(&tomorrow));
            app.set_week_weather(to_slint_week(&week));
            app.set_loading(false);
            tracing::info!(
                today_temp = today.temperature.current,
                tomorrow_temp = tomorrow.temperature.current,
                week_days = week.len(),
                "Weather data fetched successfully"
            );
        }
        Err(e) => {
            app.set_loading(false);
            app.set_error_message(format!("Failed to load weather data: {}", e).into());
            tracing::error!(error = %e, "Failed to fetch weather data");
        }
    }

    // Show the window
    app.show()?;

    // Run the Slint event loop
    slint::run_event_loop()?;
    Ok(())
}
