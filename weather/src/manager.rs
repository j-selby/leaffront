/// The weather manager controls a weather polling thread, and provides a mechanism to poll
/// for weather whenever required.

use ::{Weather, WeatherProviderKind};

use WeatherProvider;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, RecvTimeoutError};

use std::thread;

use std::time::Duration;

use openweathermap::OpenWeatherMap;

use bom::BOM;

struct WeatherWorker {
    channel_sender : Sender<()>,
    channel_receiver : Receiver<Result<Weather, String>>
}

impl WeatherWorker {
    pub fn send_request(&self) {
        self.channel_sender.send(())
            .expect("Failed to send message!")
    }

    pub fn wait_for_request(&self, timeout : Duration) -> Result<Result<Weather, String>,
        RecvTimeoutError> {
        self.channel_receiver.recv_timeout(timeout)
    }

    pub fn new(kind : WeatherProviderKind, config : Option<toml::Value>) -> Self {
        let (request_tx, request_rx) = mpsc::channel();
        let (response_tx, response_rx) = mpsc::channel();

        thread::spawn(move || {
            'main_loop:
            loop {
                match request_rx.recv() {
                    Ok(_) => {
                        // We have a weather request - service it.
                        let weather = match kind {
                            WeatherProviderKind::OpenWeatherMap => OpenWeatherMap::get_weather(config.clone()),
                            WeatherProviderKind::BOM =>  BOM::get_weather(config.clone()),
                        };
                        response_tx.send(weather)
                            .expect("Failed to send weather to weather control thread");
                    },
                    Err(_) => {
                        // We were disconnected?
                        break 'main_loop;
                    },
                }
            }
        });

        WeatherWorker {
            channel_sender: request_tx,
            channel_receiver: response_rx
        }
    }
}

pub struct WeatherManager {
    input: Receiver<Result<Weather, String>>,
    current: Option<Result<Weather, String>>,
}

impl WeatherManager {
    /// Gets the latest weather information.
    pub fn get(&mut self) -> Result<Weather, String> {
        for result in self.input.try_iter() {
            self.current = Some(result);
        }

        let data = self.current.clone();

        match data {
            Some(weather) => weather,
            None => Err("unavailable".into()),
        }
    }

    /// Creates a new manager with a dedicated thread.
    /// update_frequency: milliseconds between updates
    pub fn new(update_frequency: u64, provider : WeatherProviderKind, config : Option<toml::Value>) -> Self {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let mut worker = WeatherWorker::new(provider, config.clone());

            loop {
                worker.send_request();

                match worker.wait_for_request(Duration::from_secs(10)) {
                    // If polling worked fine
                    Ok(weather) => {
                        let success = weather.is_ok();

                        match &weather {
                            &Err(ref x) => {
                                println!("Weather update failed ({:?}); retrying in 10 seconds...", x);
                            }
                            _ => {}
                        }

                        tx.send(weather).expect("Failed to send weather to main thread");

                        if success {
                            thread::sleep(Duration::from_millis(update_frequency));
                        } else {
                            thread::sleep(Duration::from_millis(10 * 1000));
                        }
                    },
                    Err(e) => {
                        println!("Weather thread timed out ({:?}); \
                        reinitialising and retrying in 10 seconds...", e);
                        thread::sleep(Duration::from_millis(10 * 1000));

                        worker = WeatherWorker::new(provider, config.clone());
                    }
                }
            }
        });

        WeatherManager {
            input: rx,
            current: None,
        }
    }
}
