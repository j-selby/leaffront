//! Defines basic types about weather

extern crate reqwest;
#[macro_use]
extern crate serde;
extern crate toml;
extern crate inflector;

pub mod openweathermap;
pub mod manager;

#[derive(Clone)]
pub struct Weather {
    pub temperature: f64,
    pub description: String,
}

pub trait WeatherProvider {
    fn get_weather(config : Option<toml::Value>) -> Result<Weather, String>;
}