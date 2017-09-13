/// The weather manager controls a weather polling thread, and provides a mechanism to poll
/// for weather whenever required.

use weather::Weather;
use weather::WeatherProvider;
use weather::bom::BOM;

use std::sync::mpsc::{Sender, Receiver, TryRecvError};
use std::sync::mpsc;

use std::thread;

use std::time::Duration;

pub struct WeatherManager {
    input   : Receiver<Result<Weather, String>>,
    current : Option<Result<Weather, String>>
}

impl WeatherManager {
    /// Gets the latest weather information.
    pub fn get(&mut self) -> Result<Weather, String> {
        let result = self.input.try_recv();
        let data = match result {
            Ok(result) => {
                self.current = Some(result);
                self.current.clone()
            },
            Err(TryRecvError::Empty) => {
                self.current.clone()
            },
            Err(TryRecvError::Disconnected) => {
                println!("Disconnected?");
                self.current.clone()
            }
        };

        match data {
            Some(weather) => {
                weather
            },
            None => {
                Err("unavailable".into())
            }
        }
    }

    /// Creates a new manager with a dedicated thread.
    /// update_frequency: milliseconds between updates
    pub fn new(update_frequency : u64) -> Self {
        let (tx, rx): (Sender<Result<Weather, String>>,
                       Receiver<Result<Weather, String>>) = mpsc::channel();

        thread::spawn(move || {
            loop {
                let weather = BOM::get_weather();
                let success = weather.is_ok();
                tx.send(weather).unwrap();
                if success {
                    thread::sleep(Duration::from_millis(update_frequency));
                } else {
                    println!("Weather update failed; retrying in 10 seconds...");
                    thread::sleep(Duration::from_millis(10 * 1000));
                }
            }
        });

        WeatherManager {
            input : rx,
            current : None
        }
    }
}
