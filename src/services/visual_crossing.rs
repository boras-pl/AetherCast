// Copyright © 2026 AetherCast Contributors
// SPDX-License-Identifier: MIT

//! Visual Crossing API adapter.
//!
//! 1000 requests/day free tier.
//! Requires an API key. Documentation: https://www.visualcrossing.com/

use crate::models::weather::{
    WeatherCondition, WeatherData, Temperature, condition_to_icon,
    wmo_code_to_condition, wind_direction_from_angle,
};
use crate::services::weather_service::{WeatherService, WeatherError};
use chrono::NaiveDate;
use serde::Deserialize;

pub struct VisualCrossingProvider {
    client: reqwest::Client,
    api_key: String,
}

impl VisualCrossingProvider {
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
        std::env::var("VISUALCROSSING_API_KEY").ok().map(|key| Self::new(key))
    }
}

#[async_trait::async_trait]
impl WeatherService for VisualCrossingProvider {
    fn name(&self) -> &str {
        "Visual Crossing"
    }

    async fn fetch(
        &self,
        lat: f64,
        lon: f64,
    ) -> Result<(WeatherData, WeatherData, Vec<WeatherData>), WeatherError> {
        let url = format!(
            "https://weather.visualcrossing.com/VisualCrossingWebServices/rest/services/timeline/{},{}/next-8days?unitGroup=metric&key={}&include=current,days",
            lat, lon, self.api_key
        );

        tracing::debug!(url = %url, "Fetching weather from Visual Crossing");

        let response = self.client.get(&url).send().await?;
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(WeatherError::ProviderError(
                format!("Visual Crossing error ({}): {}", status, text),
            ));
        }

        let body: VisualCrossingResponse = response.json().await?;

        let current = parse_current(&body)?;

        let mut tomorrow_weather: Option<WeatherData> = None;
        let mut weekly: Vec<WeatherData> = Vec::new();

        for day in &body.days {
            let date = NaiveDate::parse_from_str(&day.datetime, "%Y-%m-%dT%H:%M:%S")
                .or_else(|_| day.datetime.split(' ').next().unwrap_or("").parse())
                .unwrap_or_else(|_| chrono::Local::now().date_naive());

            let weather = WeatherData {
                date,
                temperature: Temperature {
                    current: day.temp,
                    min: day.tomin,
                    max: day.temax,
                    unit: crate::models::weather::TemperatureUnit::Celsius,
                },
                pressure: day.pressure,
                wind_speed: day.windspeed,
                wind_direction: wind_direction_from_angle(day.winddir),
                humidity: day.humidity as u8,
                condition: WeatherCondition::Unknown(day.conditions.clone()),
                description: day.conditions.clone(),
                icon_code: "unknown".to_string(),
                feels_like: day.feelslike,
                uv_index: None,
                precipitation: day.precip,
                visibility: None,
            };

            if date == chrono::Local::now().date_naive() + chrono::Duration::days(1) {
                tomorrow_weather = Some(weather);
            } else {
                weekly.push(weather);
            }
        }

        let tomorrow = tomorrow_weather.unwrap_or_else(|| create_fallback());

        tracing::info!(provider = "visual_crossing", "Successfully fetched weather data");

        Ok((current, tomorrow, weekly))
    }
}

fn parse_current(body: &VisualCrossingResponse) -> Result<WeatherData, WeatherError> {
    let cur = &body.currentConditions;

    Ok(WeatherData {
        date: chrono::Local::now().date_naive(),
        temperature: Temperature {
            current: cur.temp,
            min: cur.temp - 3.0,
            max: cur.temp + 3.0,
            unit: crate::models::weather::TemperatureUnit::Celsius,
        },
        pressure: cur.pressure,
        wind_speed: cur.windspeed,
        wind_direction: wind_direction_from_angle(cur.winddir),
        humidity: cur.humidity as u8,
        condition: WeatherCondition::Unknown(cur.conditions.clone()),
        description: cur.conditions.clone(),
        icon_code: "unknown".to_string(),
        feels_like: cur.feelslike,
        uv_index: None,
        precipitation: 0.0,
        visibility: None,
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
struct VisualCrossingResponse {
    currentConditions: VisualCrossingCurrent,
    days: Vec<VisualCrossingDay>,
}

#[derive(Debug, Deserialize)]
struct VisualCrossingCurrent {
    temp: f32,
    feelslike: f32,
    pressure: f32,
    humidity: f32,
    windspeed: f32,
    winddir: f32,
    conditions: String,
}

#[derive(Debug, Deserialize)]
struct VisualCrossingDay {
    datetime: String,
    temp: f32,
    tomin: f32,
    temax: f32,
    feelslike: f32,
    pressure: f32,
    humidity: f32,
    windspeed: f32,
    winddir: f32,
    precip: f32,
    conditions: String,
}
