// Copyright © 2026 AetherCast Contributors
// SPDX-License-Identifier: MIT

//! Mock / offline weather data provider.
//!
//! Returns deterministic sample data for testing the UI without network access.
//! This is the last-resort fallback when all real providers fail.

use crate::models::weather::{
    WeatherCondition, WeatherData, Temperature,
};
use crate::services::weather_service::{WeatherService, WeatherError};
use chrono::{Days, Local};

pub struct MockProvider;

impl MockProvider {
    pub fn new() -> Self {
        Self
    }

    fn generate_weather(&self, date: chrono::NaiveDate, seed: u64) -> WeatherData {
        let conditions = [
            WeatherCondition::ClearSky,
            WeatherCondition::PartlyCloudy,
            WeatherCondition::Overcast,
            WeatherCondition::Rain,
            WeatherCondition::Snow,
            WeatherCondition::Thunderstorm,
            WeatherCondition::Fog,
        ];

        let condition = &conditions[(seed as usize % conditions.len())];
        let base_temp = 10.0 + (seed as f32 % 20.0) - 10.0;

        WeatherData {
            date,
            temperature: Temperature {
                current: base_temp,
                min: base_temp - 5.0,
                max: base_temp + 5.0,
                unit: crate::models::weather::TemperatureUnit::Celsius,
            },
            pressure: 1000.0 + (seed as f32 % 30.0),
            wind_speed: 5.0 + (seed as f32 % 30.0),
            wind_direction: ["N", "NE", "E", "SE", "S", "SW", "W", "NW"]
                [(seed as usize % 8)].to_string(),
            humidity: 40 + (seed as u8 % 50),
            condition: condition.clone(),
            description: format!("Mock data for {}", date),
            icon_code: crate::models::weather::condition_to_icon(condition).to_string(),
            feels_like: base_temp - 2.0,
            uv_index: Some(3.0 + (seed as f32 % 7.0)),
            precipitation: (seed as f32 % 10.0),
            visibility: Some(8.0 + (seed as f32 % 12.0)),
        }
    }
}

#[async_trait::async_trait]
impl WeatherService for MockProvider {
    fn name(&self) -> &str {
        "Mock"
    }

    async fn fetch(
        &self,
        _lat: f64,
        _lon: f64,
    ) -> Result<(WeatherData, WeatherData, Vec<WeatherData>), WeatherError> {
        let today = Local::now().date_naive();
        let tomorrow = today + Days::new(1);

        let today_data = self.generate_weather(today, 42);
        let tomorrow_data = self.generate_weather(tomorrow, 137);

        let mut weekly = Vec::new();
        for i in 2..=8 {
            let date = today + Days::new(i as u64);
            weekly.push(self.generate_weather(date, i as u64 * 100 + 7));
        }

        tracing::info!(provider = "mock", "Using mock weather data (offline mode)");

        Ok((today_data, tomorrow_data, weekly))
    }
}
