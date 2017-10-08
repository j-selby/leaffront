extern crate ftp;
extern crate xmltree;

/// Defines basic types about weather

pub mod bom;
pub mod manager;

#[derive(Clone)]
pub struct Weather {
    pub temperature: f64,
    pub description: String
}

pub trait WeatherProvider {
    fn get_weather() -> Result<Weather, String>;
}
