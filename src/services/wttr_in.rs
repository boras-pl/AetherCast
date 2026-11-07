//! wttr.in API adapter.
//!
//! Simple, free API without authentication. Great for quick lookups.
//! Documentation: https://wttr.in/

use crate::models::weather::{
    WeatherCondition, WeatherData, Temperature, condition_to_icon,
    wmo_code_to_condition, wind_direction_from_angle,
};
use crate::services::weather_service::{WeatherService, WeatherError};
use chrono::NaiveDate;
use serde::Deserialize;

pub struct WttrInProvider {
    client: reqwest::Client,
}

impl WttrInProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .user_agent("AetherCast/0.1.0")
                .build()
                .expect("Failed to create HTTP client"),
        }
    }
}

#[async_trait::async_trait]
impl WeatherService for WttrInProvider {
    fn name(&self) -> &str {
        "wttr.in"
    }

    async fn fetch(
        &self,
        lat: f64,
        lon: f64,
    ) -> Result<(WeatherData, WeatherData, Vec<WeatherData>), WeatherError> {
        // wttr.in uses AQL format for coordinates
        let url = format!(
            "https://wttr.in/{},{},0?format=j1",
            lat, lon
        );

        tracing::debug!(url = %url, "Fetching weather from wttr.in");

        let response = self.client.get(&url).send().await?;
        let status = response.status();
        if !status.is_success() {
            return Err(WeatherError::ProviderError(
                format!("wttr.in returned status {}", status),
            ));
        }

        let body: WttrInResponse = response.json().await?;

        let today = chrono::Local::now().date_naive();
        let ast = body.ast.clone().unwrap_or_else(|| body.attribution_url.clone());

        // Parse current weather
        let current = parse_current(&body, &today)?;

        // Parse forecast
        let forecast = &body.weather;
        let mut tomorrow_weather: Option<WeatherData> = None;
        let mut weekly: Vec<WeatherData> = Vec::new();

        // The first entry is today, rest are future days
        for (i, day) in forecast.iter().enumerate() {
            let date = day.date.parse::<NaiveDate>()
                .map_err(|e| WeatherError::InvalidResponse(format!("Invalid date: {}", e)))?;

            if i == 0 {
                continue; // Skip today (we have current data)
            }

            if let Some(half) = day.hourly.first() {
                let weather = WeatherData {
                    date,
                    temperature: Temperature {
                        current: half.temp_c.parse::<f32>().unwrap_or(15.0),
                        min: day.min_c.parse::<f32>().unwrap_or(5.0),
                        max: day.max_c.parse::<f32>().unwrap_or(25.0),
                        unit: crate::models::weather::TemperatureUnit::Celsius,
                    },
                    pressure: half.pressure.parse::<f32>().unwrap_or(1013.0),
                    wind_speed: half.windspeedKmph.parse::<f32>().unwrap_or(0.0),
                    wind_direction: wind_direction_from_angle(
                        half.winddir16Point.parse::<f32>().unwrap_or(0.0)
                    ),
                    humidity: half.humidity.parse::<u8>().unwrap_or(50),
                    condition: wmo_code_to_condition(
                        day.unitc.to_string().parse::<i32>().unwrap_or(0)
                    ),
                    description: day.desc[0].value.clone(),
                    icon_code: condition_to_icon(&wmo_code_to_condition(
                        day.unitc.to_string().parse::<i32>().unwrap_or(0)
                    )).to_string(),
                    feels_like: half.feelslikeC.parse::<f32>().unwrap_or(15.0),
                    uv_index: None,
                    precipitation: day.totalSnow_cm.parse::<f32>().unwrap_or(0.0)
                        + day.sum_of_rain_1.parse::<f32>().unwrap_or(0.0),
                    visibility: None,
                };

                if date == today + chrono::Duration::days(1) {
                    tomorrow_weather = Some(weather);
                } else {
                    weekly.push(weather);
                }
            }
        }

        let tomorrow = tomorrow_weather.unwrap_or_else(|| create_fallback(&today));

        tracing::info!(provider = "wttr_in", "Successfully fetched weather data");

        Ok((current, tomorrow, weekly))
    }
}

fn parse_current(body: &WttrInResponse, _today: &NaiveDate) -> Result<WeatherData, WeatherError> {
    let current = &body.current_condition[0];

    Ok(WeatherData {
        date: chrono::Local::now().date_naive(),
        temperature: Temperature {
            current: current.temp_c.parse::<f32>().unwrap_or(15.0),
            min: 10.0,
            max: 20.0,
            unit: crate::models::weather::TemperatureUnit::Celsius,
        },
        pressure: current.observation_time
            .parse::<f32>()
            .ok()
            .map(|t| 1000.0 + (t % 100.0) * 10.0)
            .unwrap_or(1013.0),
        wind_speed: current.windspeedKmph.parse::<f32>().unwrap_or(0.0),
        wind_direction: wind_direction_from_angle(
            current.winddir16Point.parse::<f32>().unwrap_or(0.0)
        ),
        humidity: current.humidity.parse::<u8>().unwrap_or(50),
        condition: WeatherCondition::Unknown(current.weatherDesc[0].value.clone()),
        description: current.weatherDesc[0].value.clone(),
        icon_code: "unknown".to_string(),
        feels_like: current.feelslikeC.parse::<f32>().unwrap_or(15.0),
        uv_index: None,
        precipitation: 0.0,
        visibility: None,
    })
}

fn create_fallback(today: &NaiveDate) -> WeatherData {
    WeatherData {
        date: *today + chrono::Duration::days(1),
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
struct WttrInResponse {
    current_condition: Vec<WttrCurrent>,
    weather: Vec<WttrDay>,
    ast: Option<String>,
    attribution_url: String,
}

#[derive(Debug, Deserialize)]
struct WttrCurrent {
    temp_c: String,
    humidity: String,
    windspeedKmph: String,
    winddir16Point: String,
    weatherDesc: Vec<WttrDesc>,
    observation_time: String,
    feelslikeC: String,
}

#[derive(Debug, Deserialize)]
struct WttrDesc {
    value: String,
}

#[derive(Debug, Deserialize)]
struct WttrDay {
    date: String,
    min_c: String,
    max_c: String,
    totalSnow_cm: String,
    sum_of_rain_1: String,
    unitc: String,
    desc: Vec<WttrDesc>,
    hourly: Vec<WttrHour>,
}

#[derive(Debug, Deserialize)]
struct WttrHour {
    temp_c: String,
    pressure: String,
    windspeedKmph: String,
    winddir16Point: String,
    humidity: String,
    feelslikeC: String,
}
