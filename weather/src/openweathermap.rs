//! Fetches weather from OpenWeatherMap
#![allow(dead_code)]

use Weather;
use WeatherProvider;

use inflector::Inflector;

static ENDPOINT: &'static str = "https://api.openweathermap.org/data/2.5/weather";

/// Expected temperature output units
#[derive(Deserialize, Debug)]
enum WeatherUnits {
    Kelvin,
    Metric,
    Fahrenheit,
}

/// Configuration for modifying the response from OpenWeatherMap
#[derive(Deserialize, Debug)]
struct OpenWeatherMapConfig {
    api_key: String,
    // e.g. "Sydney,AU"
    location: String,
    temp_units: WeatherUnits,
}

/// Actual location for request
#[derive(Deserialize, Debug)]
struct ResponseCoords {
    #[serde(default)]
    lon: f64,
    #[serde(default)]
    lat: f64,
}

/// Contains a particular state of weather (e.g. "cloudy")
#[derive(Deserialize, Debug)]
struct ResponseWeatherEntry {
    #[serde(default)]
    id: u64,
    #[serde(default)]
    main: String,
    description: String,
    #[serde(default)]
    icon: String,
}

/// Contains the main set of measurements (temp, humidity, etc)
#[derive(Deserialize, Debug)]
struct ResponseWeatherMainMeasurements {
    temp: f64,
    #[serde(default)]
    pressure: f64,
    #[serde(default)]
    humidity: f64,
    #[serde(default)]
    temp_min: f64,
    #[serde(default)]
    temp_max: f64,
}

/// Direction/speed of wind
#[derive(Deserialize, Debug)]
struct ResponseWeatherWind {
    speed: f64,
    deg: f64,
}

/// Information about cloud levels
#[derive(Deserialize, Debug)]
struct ResponseWeatherClouds {
    #[serde(default)]
    all: f64,
}

/// Information about the location of the data collector
#[derive(Deserialize, Debug)]
struct ResponseWeatherSystem {
    #[serde(rename = "type")]
    #[serde(default)]
    sys_type: u64,
    #[serde(default)]
    id: u64,
    #[serde(default)]
    country: String,
    #[serde(default)]
    sunrise: u64,
    #[serde(default)]
    sunset: u64,
}

/// JSON output from OpenWeatherMap
#[derive(Deserialize)]
struct OpenWeatherMapResponse {
    #[serde(default)]
    coord: Option<ResponseCoords>,
    weather: Vec<ResponseWeatherEntry>,
    /// Unknown use - reports "stations"?
    #[serde(default)]
    base: Option<String>,
    main: ResponseWeatherMainMeasurements,
    #[serde(default)]
    visibility: Option<u64>,
    #[serde(default)]
    wind: Option<ResponseWeatherWind>,
    #[serde(default)]
    clouds: Option<ResponseWeatherClouds>,
    // Day/time
    #[serde(default)]
    dt: Option<u64>,
    #[serde(default)]
    sys: Option<ResponseWeatherSystem>,
    #[serde(default)]
    timezone: Option<u64>,
    #[serde(default)]
    id: Option<u64>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    cod: Option<u64>,
}

pub struct OpenWeatherMap;

impl OpenWeatherMap {}

impl WeatherProvider for OpenWeatherMap {
    fn get_weather(config: Option<toml::Value>) -> Result<Weather, String> {
        // We require a config for OpenWeatherMap:
        let config = config.ok_or_else(|| "OpenWeatherMap configuration needed".to_string())?;

        // Parse into a configuration type
        let config: OpenWeatherMapConfig = config
            .try_into()
            .map_err(|x| format!("Failed to parse OpenWeatherMap config: {:?}", x))?;

        let client = reqwest::blocking::Client::new();

        let mut request = client
            .get(ENDPOINT)
            .query(&[("APPID", config.api_key), ("q", config.location)]);

        match config.temp_units {
            WeatherUnits::Kelvin => {
                // By default
            }
            WeatherUnits::Metric => {
                request = request.query(&[("units", "metric")]);
            }
            WeatherUnits::Fahrenheit => {
                request = request.query(&[("units", "imperial")]);
            }
        }

        let response = request
            .send()
            .map_err(|x| format!("Failed to get weather status: {:?}", x))?;

        response
            .error_for_status_ref()
            .map_err(|x| format!("Got bad status code while getting weather: {:?}", x))?;

        let json: OpenWeatherMapResponse = response
            .json()
            .map_err(|x| format!("Failed to parse weather JSON: {:?}", x))?;

        info!("Downloaded weather from OpenWeatherMap successfully");

        let weather_state = json
            .weather
            .get(0)
            .ok_or_else(|| "No weather entry in JSON response".to_string())?;

        let mut description = weather_state.description.to_sentence_case();

        if !description.ends_with(".") {
            description += ".";
        }

        let weather = Weather {
            temperature: json.main.temp,
            description: description,
        };

        Ok(weather)
    }
}
