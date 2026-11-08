// Copyright © 2026 AetherCast Contributors
// SPDX-License-Identifier: MIT

//! Weather services module.
//!
//! Defines the trait for weather providers and the fallback chain.

pub mod weather_service;
pub mod open_meteo;
pub mod wttr_in;
pub mod open_weather_map;
pub mod weather_api;
pub mod visual_crossing;
pub mod mock;

pub use weather_service::{WeatherService, WeatherError};
pub use mock::MockProvider;
pub use open_meteo::OpenMeteoProvider;
pub use wttr_in::WttrInProvider;
pub use open_weather_map::OpenWeatherMapProvider;
pub use weather_api::WeatherApiProvider;
pub use visual_crossing::VisualCrossingProvider;
