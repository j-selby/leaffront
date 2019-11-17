//! A BOM (Australia) frontend for the Weather API.

use ::{WeatherProvider, Weather};

use reqwest::header;
use serde::Deserialize;

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

fn try_with_different_length_geocodes<T, F>(client : &reqwest::Client, endpoint : F, geohash : &str) -> Result<T, String>
    where F: Fn(&str) -> String,
          for<'de> T: serde::Deserialize<'de> {
    match client.get(&endpoint(geohash))
        .send()
        .map_err(|x| format!("Error sending request: {:?}", x))?
        .json()
        .map_err(|x| format!("Error parsing request: {:?}", x)) {
        Ok(v) => Ok(v),
        Err(e) => {
            if geohash.len() < 7 {
                return Err(e);
            }

            println!("BOM API: Failed to parse forecasts with 7-code geohash ({:?}), retrying with 6...", e);

            let short_geocode = &geohash[0..6];

            assert_eq!(short_geocode.len(), 6);

            client.get(&endpoint(short_geocode))
                .send()
                .map_err(|x| format!("Error sending request: {:?}", x))?
                .json()
                .map_err(|x| format!("Error parsing request: {:?}", x))
        }
    }
}

impl WeatherProvider for BOM {
    fn get_weather(config: Option<toml::Value>) -> Result<Weather, String> {
        // We require a config for BOM:
        let config = config
            .ok_or_else(|| "BOM configuration needed".to_string())?;

        // Parse into a configuration type
        let config : BOMConfig = config.try_into()
            .map_err(|x| format!("Failed to parse BOM config: {:?}", x))?;

        let mut headers = header::HeaderMap::new();
        headers.insert(header::USER_AGENT,
                       header::HeaderValue::from_static(
                           concat!(
                            "LeafFront/v",
                            env!("CARGO_PKG_VERSION"),
                            " (https://github.com/j-selby/leaffront)"
                           )
                       )
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|x| format!("Failed to build reqwest client: {:?}", x))?;

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

        // Attempt with the full geohash, then retry with regional info
        let weather_response : ResponseForecast =
            try_with_different_length_geocodes(&client,
                                               |geohash| {
                                                   format!("{}/locations/{}/forecasts/daily", ENDPOINT, geohash)
                                               },
                                               &location_info.geohash)
            .map_err(|x| format!("Failed to download BOM forecasts weather response: {:?}", x))?;

        // Attempt to get the first element of the forecast
        let weather_entry = weather_response.data.get(0)
            .ok_or_else(|| format!("No weather for location of {:?}", config.location))?;

        let description = weather_entry.short_text.clone()
            .ok_or_else(|| format!("No current weather description for location of {:?}", config.location))?;

        let observations_response : ResponseObservations =
            try_with_different_length_geocodes(&client,
                                               |geohash| {
                                                   format!("{}/locations/{}/observations", ENDPOINT, geohash)
                                               },
                                               &location_info.geohash)
                .map_err(|x| format!("Failed to download BOM forecasts weather response: {:?}", x))?;

        println!("Downloaded weather from BOM successfully");

        Ok (
            Weather {
                temperature : observations_response.data.temp,
                description,
            }
        )
    }
}
