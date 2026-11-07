//! Weather service trait and error types.
//!
//! Defines the abstraction for weather data providers.

use crate::models::weather::WeatherData;
use thiserror::Error;

/// Errors that can occur during weather data fetching.
#[derive(Debug, Error)]
pub enum WeatherError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Provider not available: {0}")]
    ProviderError(String),

    #[error("Invalid response from provider: {0}")]
    InvalidResponse(String),

    #[error("Network timeout")]
    Timeout,
}

/// Trait for weather data providers.
///
/// Each provider implements this trait to fetch weather data
/// from their respective API.
#[async_trait::async_trait]
pub trait WeatherService: Send + Sync {
    /// Returns the name of this provider (for logging).
    fn name(&self) -> &str;

    /// Fetches current weather and forecast data.
    ///
    /// # Arguments
    /// * `lat` — Latitude
    /// * `lon` — Longitude
    ///
    /// # Returns
    /// Tuple of (today's weather, tomorrow's weather, weekly forecast)
    async fn fetch(
        &self,
        lat: f64,
        lon: f64,
    ) -> Result<(WeatherData, WeatherData, Vec<WeatherData>), WeatherError>;
}

/// A provider that has been tried and failed.
#[derive(Debug)]
pub struct FailedProvider {
    pub name: String,
    pub error: WeatherError,
}
