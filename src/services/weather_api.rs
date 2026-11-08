// Copyright © 2026 AetherCast Contributors
// SPDX-License-Identifier: MIT

//! WeatherAPI.com API adapter.
//!
//! Good free tier (1M requests/month).
//! Requires an API key. Documentation: https://www.weatherapi.com/

use crate::models::weather::{
    WeatherCondition, WeatherData, Temperature, condition_to_icon,
    wmo_code_to_condition, wind_direction_from_angle,
};
use crate::services::weather_service::{WeatherService, WeatherError};
use chrono::NaiveDate;
use serde::Deserialize;

pub struct WeatherApiProvider {
    client: reqwest::Client,
    api_key: String,
}

impl WeatherApiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            api_key,
        }
    }

    pub fn from_env() -> Option<Self> {
        std::env::var("WEATHERAPI_KEY").ok().map(|key| Self::new(key))
    }
}

#[async_trait::async_trait]
impl WeatherService for WeatherApiProvider {
    fn name(&self) -> &str {
        "WeatherAPI.com"
    }

    async fn fetch(
        &self,
        lat: f64,
        lon: f64,
    ) -> Result<(WeatherData, WeatherData, Vec<WeatherData>), WeatherError> {
        let url = format!(
            "http://api.weatherapi.com/v1/forecast.json?key={}&q={},{}&days=8&aqi=no",
            self.api_key, lat, lon
        );

        tracing::debug!(url = %url, "Fetching weather from WeatherAPI.com");

        let response = self.client.get(&url).send().await?;
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(WeatherError::ProviderError(
                format!("WeatherAPI error ({}): {}", status, text),
            ));
        }

        let body: WeatherApiResponse = response.json().await?;

        let current = parse_current(&body)?;
        let forecast = &body.forecast.forecastday;

        let mut tomorrow_weather: Option<WeatherData> = None;
        let mut weekly: Vec<WeatherData> = Vec::new();

        for day in forecast {
            let date = day.date.parse::<NaiveDate>()
                .map_err(|e| WeatherError::InvalidResponse(format!("Invalid date: {}", e)))?;

            let weather = WeatherData {
                date,
                temperature: Temperature {
                    current: day.day.avgtemp_c,
                    min: day.day.mintemp_c,
                    max: day.day.maxtemp_c,
                    unit: crate::models::weather::TemperatureUnit::Celsius,
                },
                pressure: day.day.pressure_mb,
                wind_speed: day.day.maxwind_kph,
                wind_direction: wind_direction_from_angle(day.day.wind_degree as f32),
                humidity: day.day.avghumidity as u8,
                condition: wmo_code_to_condition(day.day.condition.code),
                description: day.day.condition.text.clone(),
                icon_code: condition_to_icon(&wmo_code_to_condition(day.day.condition.code)).to_string(),
                feels_like: day.day.maxfeelslike_c,
                uv_index: Some(day.day.uv),
                precipitation: day.day.totalprecip_mm,
                visibility: Some(day.day.avgvis_km),
            };

            if date == chrono::Local::now().date_naive() + chrono::Duration::days(1) {
                tomorrow_weather = Some(weather);
            } else {
                weekly.push(weather);
            }
        }

        let tomorrow = tomorrow_weather.unwrap_or_else(|| create_fallback());

        tracing::info!(provider = "weather_api", "Successfully fetched weather data");

        Ok((current, tomorrow, weekly))
    }
}

fn parse_current(body: &WeatherApiResponse) -> Result<WeatherData, WeatherError> {
    let cur = &body.current;
    let loc = &body.location;

    Ok(WeatherData {
        date: chrono::Local::now().date_naive(),
        temperature: Temperature {
            current: cur.temp_c,
            min: cur.temp_c - 3.0,
            max: cur.temp_c + 3.0,
            unit: crate::models::weather::TemperatureUnit::Celsius,
        },
        pressure: cur.pressure_mb,
        wind_speed: cur.wind_kph,
        wind_direction: wind_direction_from_angle(cur.wind_degree as f32),
        humidity: cur.humidity as u8,
        condition: wmo_code_to_condition(cur.condition.code),
        description: cur.condition.text.clone(),
        icon_code: condition_to_icon(&wmo_code_to_condition(cur.condition.code)).to_string(),
        feels_like: cur.feelslike_c,
        uv_index: Some(cur.uv),
        precipitation: 0.0,
        visibility: Some(cur.vis_km),
    })
}

fn create_fallback() -> WeatherData {
    WeatherData {
        date: chrono::Local::now().date_naive() + chrono::Duration::days(1),
        temperature: Temperature {
            current: 15.0, min: 8.0, max: 20.0,
            unit: crate::models::weather::TemperatureUnit::Celsius,
        },
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
    }
}

#[derive(Debug, Deserialize)]
struct WeatherApiResponse {
    current: WeatherApiCurrent,
    location: WeatherApiLocation,
    forecast: WeatherApiForecast,
}

#[derive(Debug, Deserialize)]
struct WeatherApiCurrent {
    temp_c: f32,
    feelslike_c: f32,
    pressure_mb: f32,
    humidity: u16,
    wind_kph: f32,
    wind_degree: u16,
    vis_km: f32,
    uv: f32,
    condition: WeatherApiCondition,
}

#[derive(Debug, Deserialize)]
struct WeatherApiCondition {
    code: i32,
    text: String,
}

#[derive(Debug, Deserialize)]
struct WeatherApiLocation {
    name: String,
}

#[derive(Debug, Deserialize)]
struct WeatherApiForecast {
    forecastday: Vec<WeatherApiDay>,
}

#[derive(Debug, Deserialize)]
struct WeatherApiDay {
    date: String,
    day: WeatherApiDayData,
}

#[derive(Debug, Deserialize)]
struct WeatherApiDayData {
    maxtemp_c: f32,
    mintemp_c: f32,
    avgtemp_c: f32,
    maxfeelslike_c: f32,
    maxwind_kph: f32,
    avghumidity: u16,
    pressure_mb: f32,
    wind_degree: u16,
    totalprecip_mm: f32,
    avgvis_km: f32,
    uv: f32,
    condition: WeatherApiCondition,
}
