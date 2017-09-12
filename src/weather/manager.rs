/// The weather manager controls a weather polling thread, and provides a mechanism to poll
/// for weather whenever required.

use weather::Weather;
use weather::WeatherProvider;
use weather::bom::BOM;

use std::sync::mpsc::{Sender, Receiver, TryRecvError};
use std::sync::mpsc;

use std::thread;
use std::thread::JoinHandle;

use std::time::Duration;

pub struct Manager {
    input   : Receiver<Result<Weather, String>>,
    handle  : JoinHandle<()>,
    current : Option<Result<Weather, String>>
}

impl Manager {
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

        let handle  = thread::spawn(move || {
            loop {
                if false {
                    tx.send(BOM::get_weather()).unwrap();
                }
                thread::sleep(Duration::from_millis(update_frequency));
            }
        });

        Manager {
            input : rx,
            handle,
            current : None
        }
    }
}