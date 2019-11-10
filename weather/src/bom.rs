//! A BOM (Australia) frontend for the Weather API.

use ::{WeatherProvider, Weather};

static ENDPOINT : &'static str = "https://api.weather.bom.gov.au/v1";

#[derive(Deserialize, Debug)]
struct BOMConfig {
    /// Some location - will be looked up against BOM's API.
    /// e.g. "Sydney"
    location : String
}

/// Metadata tag on JSON responses
#[derive(Deserialize, Debug)]
struct ResponseMetadata {
    #[serde(default)]
    response_timestamp : Option<String>,
    #[serde(default)]
    issue_time : Option<String>,
    #[serde(default)]
    forecast_region : Option<String>,
    #[serde(default)]
    forecast_type : Option<String>
}

/// Information on a single location
#[derive(Deserialize, Debug)]
struct ResponseLocation {
    geohash : String,
    #[serde(default)]
    id :  Option<String>,
    name : String,
    #[serde(default)]
    postcode : Option<String>,
    #[serde(default)]
    state : Option<String>
}

/// Response from the V1 locations API
#[derive(Deserialize, Debug)]
struct ResponseLocations {
    metadata : ResponseMetadata,
    data : Vec<ResponseLocation>
}

/// Live data as part of the first payload of forecasts
#[derive(Deserialize, Debug)]
struct ResponseWeatherNow {
    #[serde(default)]
    is_night : bool,
    now_label : String,
    #[serde(default)]
    later_label : Option<String>,
    temp_now : f64,
    #[serde(default)]
    temp_later : Option<f64>
}

/// A single weather response from the API
#[derive(Deserialize, Debug)]
struct ResponseWeather {
    // "rain", "uv", "astronomical" ignored
    date : String,
    #[serde(default)]
    temp_max : Option<f64>,
    #[serde(default)]
    temp_min : Option<f64>,
    #[serde(default)]
    extended_text : Option<String>,
    #[serde(default)]
    icon_descriptor : Option<String>,
    #[serde(default)]
    short_text : Option<String>,
    #[serde(default)]
    fire_danger : Option<String>,
    #[serde(default)]
    now : Option<ResponseWeatherNow>
}

/// The entire forecasts query
#[derive(Deserialize, Debug)]
struct ResponseForecast {
    metadata : ResponseMetadata,
    data : Vec<ResponseWeather>
}

/// The inner observations payload
#[derive(Deserialize, Debug)]
struct ResponseObservationsPayload {
    temp : f64,
    temp_feels_like : f64,
    // wind
    // rain_since_9am
    // humidity
    // staiton
}

/// Current observations for a location
#[derive(Deserialize, Debug)]
struct ResponseObservations {
    metadata : ResponseMetadata,
    data : ResponseObservationsPayload
}

pub struct BOM;

impl BOM {
}

impl WeatherProvider for BOM {
    fn get_weather(config: Option<toml::Value>) -> Result<Weather, String> {
        // We require a config for BOM:
        let config = config
            .ok_or_else(|| "BOM configuration needed".to_string())?;

        // Parse into a configuration type
        let config : BOMConfig = config.try_into()
            .map_err(|x| format!("Failed to parse BOM config: {:?}", x))?;

        let client = reqwest::Client::new();

        let locations_response : ResponseLocations = client.get(&format!("{}/locations", ENDPOINT))
            .query(&[("search", config.location.clone())])
            .send()
            .map_err(|x| format!("Failed to handle locations request: {:?}", x))?
            .json()
            .map_err(|x| format!("Failed to parse BOM weather response: {:?}", x))?;

        // Attempt to see if we actually have a response
        if locations_response.data.len() > 1 {
            println!("BOM API: returned multiple locations for {:?}, continuing with first:",
                     config.location);
        }

        let location_info = locations_response.data.get(0)
            .ok_or_else(|| format!("No response for location of {:?}", config.location))?;

        println!("BOM API: Got {:?} for location request of {:?}", location_info.name,
                 config.location);

        let weather_response : ResponseForecast =
            client.get(&format!("{}/locations/{}/forecasts/daily", ENDPOINT, location_info.geohash))
            .send()
            .map_err(|x| format!("Failed to handle forecasts request: {:?}", x))?
            .json()
            .map_err(|x| format!("Failed to parse BOM forecasts weather response: {:?}", x))?;

        // Attempt to get the first element of the forecast
        let weather_entry = weather_response.data.get(0)
            .ok_or_else(|| format!("No weather for location of {:?}", config.location))?;

        let observations_response : ResponseObservations =
            client.get(&format!("{}/locations/{}/observations", ENDPOINT, location_info.geohash))
                .send()
                .map_err(|x| format!("Failed to handle observations request: {:?}", x))?
                .json()
                .map_err(|x| format!("Failed to parse BOM observations weather response: {:?}", x))?;

        Ok (
            Weather {
                temperature : observations_response.data.temp,
                description: weather_entry.short_text.clone()
                    .ok_or_else(|| format!("No current weather description for location of {:?}", config.location))?,
            }
        )
    }
}
