use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

use std::thread;

use std::time::Duration;

use std::process;

use std::error::Error;

enum WatchdogError {
    Shutdown
}

pub struct Watchdog {
    output : Sender<Result<(), WatchdogError>>,
}

impl Watchdog {
    pub fn ping(&self) {
        self.output.send(Ok(())).unwrap()
    }

    fn shutdown(&self) {
        self.output.send(Err(WatchdogError::Shutdown)).unwrap()
    }

    pub fn build() -> Self {
        let (tx, rx): (Sender<Result<(), WatchdogError>>,
                       Receiver<Result<(), WatchdogError>>) = mpsc::channel();

        thread::spawn(move || {
            loop {
                match rx.recv_timeout(Duration::from_secs(2)) {
                    Ok(Ok(_)) => {},
                    Ok(Err(WatchdogError::Shutdown)) => break,
                    Err(err) => {
                        println!("Got receive error: {}", err.description());
                        println!("Watchdog fired. Shutting down...");
                        process::exit(70); // Internal software error
                    }
                }
            }
        });

        Watchdog {
            output : tx
        }
    }
}

impl Drop for Watchdog {
    fn drop(&mut self) {
        self.shutdown()
    }
}