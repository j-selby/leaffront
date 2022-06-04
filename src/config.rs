use std::fs::File;
use std::io::Read;

use toml;

use leaffront_weather::WeatherProviderKind;

#[derive(Deserialize, Debug)]
pub struct LeaffrontConfig {
    pub art_dir: String,
    pub refresh_rate: u64,
    pub sleep: Sleep,
    pub day: Day,
    pub night: Night,
    pub weather: Weather,
    pub fullscreen: bool
}

#[derive(Deserialize, Debug)]
pub struct Sleep {
    pub sleep_hour: u32,
    pub wakeup_hour: u32,
}

#[derive(Deserialize, Debug)]
pub struct Night {
    pub move_secs: u64,
    pub night_tap_cooldown: u64,
    pub brightness: u8,
}

#[derive(Deserialize, Debug)]
pub struct Day {
    pub background_secs: u64,
    pub subtitle_secs: u64,
    pub brightness: u8,
}

#[derive(Deserialize, Debug)]
pub struct Weather {
    pub update_freq: u64,
    pub kind: WeatherProviderKind,
    pub config: Option<toml::Value>,
}

/// Loads a configuration file.
pub fn load_config(dir: String) -> LeaffrontConfig {
    let mut f = File::open(dir).expect("Config file not found");

    let mut config_string = String::new();
    f.read_to_string(&mut config_string).unwrap();

    toml::from_str(&config_string).unwrap()
}
