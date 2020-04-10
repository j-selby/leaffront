//! Defines basic types about weather

extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate futures_util;
extern crate inflector;
extern crate toml;

pub mod bom;
pub mod manager;
pub mod openweathermap;

#[derive(Clone)]
pub struct Weather {
    pub temperature: f64,
    pub description: String,
}

pub trait WeatherProvider {
    fn get_weather(config: Option<toml::Value>) -> Result<Weather, String>;
}

/// What weather providers are available:
#[derive(Copy, Clone, Deserialize, Debug)]
pub enum WeatherProviderKind {
    OpenWeatherMap,
    BOM,
}
