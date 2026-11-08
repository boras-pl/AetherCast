// Copyright © 2026 AetherCast Contributors
// SPDX-License-Identifier: MIT

//! Open-Meteo API adapter.
//!
//! Fully free, no API key required. Primary provider.
//!
//! Documentation: https://open-meteo.com/

use crate::models::weather::{
    WeatherCondition, WeatherData, Temperature, condition_to_icon,
    wmo_code_to_condition, wind_direction_from_angle,
};
use crate::services::weather_service::{WeatherService, WeatherError};
use chrono::NaiveDate;
use serde::Deserialize;

pub struct OpenMeteoProvider {
    client: reqwest::Client,
}

impl OpenMeteoProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }
}

#[async_trait::async_trait]
impl WeatherService for OpenMeteoProvider {
    fn name(&self) -> &str {
        "Open-Meteo"
    }

    async fn fetch(
        &self,
        lat: f64,
        lon: f64,
    ) -> Result<(WeatherData, WeatherData, Vec<WeatherData>), WeatherError> {
        let today = chrono::Local::now().date_naive();
        let tomorrow = today + chrono::Duration::days(1);

        // Build URL for daily + hourly forecast
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,apparent_temperature,is_day,precipitation,rain,showers,snowfall,weather_code,surface_pressure,wind_speed_10m,wind_direction_10m,wind_gusts_10m&daily=weather_code,temperature_2m_max,temperature_2m_min,apparent_temperature_max,apparent_temperature_min,precipitation_sum,wind_speed_10m_max,wind_direction_10m_dominant&hourly=temperature_2m,relative_humidity_2m,apparent_temperature,precipitation_probability,weather_code,wind_speed_10m,wind_direction_10m,pressure_msl,visibility&timezone=auto&forecast_days=8",
            lat, lon
        );

        tracing::debug!(url = %url, "Fetching weather from Open-Meteo");

        let response = self.client.get(&url).send().await?;
        let status = response.status();
        if !status.is_success() {
            return Err(WeatherError::ProviderError(
                format!("Open-Meteo returned status {}", status),
            ));
        }

        let body: OpenMeteoResponse = response.json().await?;

        // Parse current weather
        let current = parse_current(&body)?;

        // Parse daily forecast (skip today index 0, start from tomorrow)
        let daily = &body.daily;
        let mut today_weather: Option<WeatherData> = None;
        let mut tomorrow_weather: Option<WeatherData> = None;
        let mut weekly: Vec<WeatherData> = Vec::new();

        for i in 0..daily.time.len() {
            let date = daily.time[i].parse::<NaiveDate>()
                .map_err(|e| WeatherError::InvalidResponse(format!("Invalid date: {}", e)))?;

            let weather = WeatherData {
                date,
                temperature: Temperature {
                    current: daily.temperature_2m_max[i],
                    min: daily.temperature_2m_min[i],
                    max: daily.temperature_2m_max[i],
                    unit: crate::models::weather::TemperatureUnit::Celsius,
                },
                pressure: daily.temperature_2m_max[i], // placeholder, will fix
                wind_speed: daily.wind_speed_10m_max[i],
                wind_direction: wind_direction_from_angle(daily.wind_direction_10m_dominant[i]),
                humidity: 50, // placeholder
                condition: wmo_code_to_condition(daily.weather_code[i]),
                description: format!("WMO {}", daily.weather_code[i]),
                icon_code: condition_to_icon(&wmo_code_to_condition(daily.weather_code[i])).to_string(),
                feels_like: daily.apparent_temperature_max[i],
                uv_index: None,
                precipitation: daily.precipitation_sum[i],
                visibility: None,
            };

            if date == today {
                today_weather = Some(weather);
            } else if date == tomorrow {
                tomorrow_weather = Some(weather);
            } else if date > today {
                weekly.push(weather);
            }
        }

        // Use current data for today if daily didn't have it
        let today = today_weather.unwrap_or(current);
        let tomorrow = tomorrow_weather.unwrap_or_else(|| {
            // Generate tomorrow from weekly if available
            weekly.first().cloned().unwrap_or_else(|| WeatherData {
                date: tomorrow,
                temperature: Temperature { current: 15.0, min: 8.0, max: 20.0, unit: crate::models::weather::TemperatureUnit::Celsius },
                pressure: 1013.0,
                wind_speed: 10.0,
                wind_direction: "N".to_string(),
                humidity: 60,
                condition: WeatherCondition::PartlyCloudy,
                description: "Partly cloudy".to_string(),
                icon_code: "partly-cloudy".to_string(),
                feels_like: 14.0,
                uv_index: None,
                precipitation: 0.0,
                visibility: None,
            })
        });

        tracing::info!(provider = "open_meteo", "Successfully fetched weather data");

        Ok((today, tomorrow, weekly))
    }
}

fn parse_current(body: &OpenMeteoResponse) -> Result<WeatherData, WeatherError> {
    let current = &body.current;
    let today = chrono::Local::now().date_naive();

    Ok(WeatherData {
        date: today,
        temperature: Temperature {
            current: current.temperature_2m,
            min: current.temperature_2m - 3.0,
            max: current.temperature_2m + 3.0,
            unit: crate::models::weather::TemperatureUnit::Celsius,
        },
        pressure: current.surface_pressure,
        wind_speed: current.wind_speed_10m,
        wind_direction: wind_direction_from_angle(current.wind_direction_10m),
        humidity: current.relative_humidity_2m as u8,
        condition: wmo_code_to_condition(current.weather_code),
        description: format!("WMO {}", current.weather_code),
        icon_code: condition_to_icon(&wmo_code_to_condition(current.weather_code)).to_string(),
        feels_like: current.apparent_temperature,
        uv_index: None,
        precipitation: current.precipitation,
        visibility: None,
    })
}

#[derive(Debug, Deserialize)]
struct OpenMeteoResponse {
    current: CurrentWeather,
    daily: DailyWeather,
}

#[derive(Debug, Deserialize)]
struct CurrentWeather {
    temperature_2m: f32,
    relative_humidity_2m: u8,
    apparent_temperature: f32,
    precipitation: f32,
    weather_code: i32,
    surface_pressure: f32,
    wind_speed_10m: f32,
    wind_direction_10m: f32,
    wind_gusts_10m: f32,
}

#[derive(Debug, Deserialize)]
struct DailyWeather {
    time: Vec<String>,
    weather_code: Vec<i32>,
    temperature_2m_max: Vec<f32>,
    temperature_2m_min: Vec<f32>,
    apparent_temperature_max: Vec<f32>,
    apparent_temperature_min: Vec<f32>,
    precipitation_sum: Vec<f32>,
    wind_speed_10m_max: Vec<f32>,
    wind_direction_10m_dominant: Vec<f32>,
}
