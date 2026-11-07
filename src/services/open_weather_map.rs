//! OpenWeatherMap API adapter.
//!
//! Popular weather API with a generous free tier (1000 calls/day).
//! Requires an API key. Documentation: https://openweathermap.org/api

use crate::models::weather::{
    WeatherCondition, WeatherData, Temperature, condition_to_icon,
    wmo_code_to_condition, wind_direction_from_angle,
};
use crate::services::weather_service::{WeatherService, WeatherError};
use chrono::NaiveDate;
use serde::Deserialize;

pub struct OpenWeatherMapProvider {
    client: reqwest::Client,
    api_key: String,
}

impl OpenWeatherMapProvider {
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
        std::env::var("OPENWEATHERMAP_API_KEY").ok().map(|key| Self::new(key))
    }
}

#[async_trait::async_trait]
impl WeatherService for OpenWeatherMapProvider {
    fn name(&self) -> &str {
        "OpenWeatherMap"
    }

    async fn fetch(
        &self,
        lat: f64,
        lon: f64,
    ) -> Result<(WeatherData, WeatherData, Vec<WeatherData>), WeatherError> {
        // One Call API 3.0 — current + daily + hourly
        let url = format!(
            "https://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&exclude=minutely,alerts&appid={}&units=metric",
            lat, lon, self.api_key
        );

        tracing::debug!(url = %url, "Fetching weather from OpenWeatherMap");

        let response = self.client.get(&url).send().await?;
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(WeatherError::ProviderError(
                format!("OpenWeatherMap error ({}): {}", status, text),
            ));
        }

        let body: OwmResponse = response.json().await?;

        // Current weather
        let current = parse_current(&body)?;

        // Daily forecast (skip today at index 0)
        let mut tomorrow_weather: Option<WeatherData> = None;
        let mut weekly: Vec<WeatherData> = Vec::new();

        for day in &body.daily {
            let date = NaiveDate::parse_from_str(&day.dt.to_string(), "%s")
                .unwrap_or_else(|_| chrono::Local::now().date_naive());

            let weather = WeatherData {
                date,
                temperature: Temperature {
                    current: day.temp.day,
                    min: day.temp.min,
                    max: day.temp.max,
                    unit: crate::models::weather::TemperatureUnit::Celsius,
                },
                pressure: day.pressure,
                wind_speed: day.wind_speed,
                wind_direction: wind_direction_from_angle(day.wind_deg),
                humidity: day.humidity as u8,
                condition: owm_code_to_condition(day.weather[0].id),
                description: day.weather[0].description.clone(),
                icon_code: condition_to_icon(&owm_code_to_condition(day.weather[0].id)).to_string(),
                feels_like: day.feels_like.day,
                uv_index: Some(day.uvi),
                precipitation: day.pop * day.rain.unwrap_or(0.0), // rough estimate
                visibility: Some(day.visibility as f32 / 1000.0),
            };

            if date == chrono::Local::now().date_naive() + chrono::Duration::days(1) {
                tomorrow_weather = Some(weather);
            } else {
                weekly.push(weather);
            }
        }

        let tomorrow = tomorrow_weather.unwrap_or_else(|| create_fallback());

        tracing::info!(provider = "open_weather_map", "Successfully fetched weather data");

        Ok((current, tomorrow, weekly))
    }
}

fn parse_current(body: &OwmResponse) -> Result<WeatherData, WeatherError> {
    let cur = &body.current;

    Ok(WeatherData {
        date: chrono::Local::now().date_naive(),
        temperature: Temperature {
            current: cur.temp,
            min: cur.temp - 3.0,
            max: cur.temp + 3.0,
            unit: crate::models::weather::TemperatureUnit::Celsius,
        },
        pressure: cur.pressure,
        wind_speed: cur.wind_speed,
        wind_direction: wind_direction_from_angle(cur.wind_deg),
        humidity: cur.humidity as u8,
        condition: owm_code_to_condition(cur.weather[0].id),
        description: cur.weather[0].description.clone(),
        icon_code: condition_to_icon(&owm_code_to_condition(cur.weather[0].id)).to_string(),
        feels_like: cur.feels_like,
        uv_index: None,
        precipitation: 0.0,
        visibility: Some(cur.visibility as f32 / 1000.0),
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

fn owm_code_to_condition(code: i32) -> WeatherCondition {
    match code {
        200..232 => WeatherCondition::Thunderstorm,
        300..322 => WeatherCondition::Drizzle,
        500..502 => WeatherCondition::Rain,
        503..504 => WeatherCondition::HeavyRain,
        511 => WeatherCondition::Drizzle,
        520..531 => WeatherCondition::Rain,
        600..622 => WeatherCondition::Snow,
        701..781 => WeatherCondition::Fog,
        800 => WeatherCondition::ClearSky,
        801 => WeatherCondition::MainlyClear,
        802 => WeatherCondition::PartlyCloudy,
        803..804 => WeatherCondition::Overcast,
        _ => WeatherCondition::Unknown(format!("OWM {}", code)),
    }
}

#[derive(Debug, Deserialize)]
struct OwmResponse {
    current: OwmCurrent,
    daily: Vec<OwmDay>,
}

#[derive(Debug, Deserialize)]
struct OwmCurrent {
    temp: f32,
    feels_like: f32,
    pressure: f32,
    humidity: u16,
    wind_speed: f32,
    wind_deg: f32,
    visibility: u32,
    weather: Vec<OwmWeather>,
    dt: u64,
}

#[derive(Debug, Deserialize)]
struct OwmDay {
    dt: u64,
    temp: OwmDayTemp,
    feels_like: OwmFeelsLike,
    pressure: f32,
    humidity: u16,
    wind_speed: f32,
    wind_deg: f32,
    weather: Vec<OwmWeather>,
    uvi: f32,
    visibility: u32,
    pop: f32,
    rain: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct OwmDayTemp {
    day: f32,
    min: f32,
    max: f32,
}

#[derive(Debug, Deserialize)]
struct OwmFeelsLike {
    day: f32,
}

#[derive(Debug, Deserialize)]
struct OwmWeather {
    id: i32,
    description: String,
}
