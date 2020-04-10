extern crate json;
extern crate leaffront_core;
extern crate redis;

use leaffront_core::backend::Backend;
use leaffront_core::backend::Notification;
use leaffront_core::version::VersionInfo;

use redis::RedisResult;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

use std::thread;

pub struct RedisBackend {
    notify: Receiver<Notification>,
}

#[derive(Debug)]
pub enum BackendError {
    RedisFail,
}

fn unwrap_redis<T>(result: RedisResult<T>) -> Result<T, BackendError> {
    match result {
        Ok(val) => Ok(val),
        Err(_) => Err(BackendError::RedisFail),
    }
}

impl RedisBackend {
    pub fn new() -> Result<Self, BackendError> {
        // TODO: Don't hardcode this URL
        let client = unwrap_redis(redis::Client::open("redis://127.0.0.1/"))?;
        let mut sub = unwrap_redis(client.get_pubsub())?;

        unwrap_redis(sub.subscribe("leaffront.notify"))?;

        // Start up listening thread
        let (notify_tx, notify_rx): (Sender<Notification>, Receiver<Notification>) =
            mpsc::channel();

        // TODO: Handle shutdowns
        thread::spawn(move || loop {
            let msg = sub.get_message().expect("Failed to parse message");
            let payload: String = msg.get_payload().expect("Failed to parse payload");
            let result = json::parse(&payload).expect("Failed to parse JSON");
            println!(
                "channel '{}': recv'd from redis: {:?}",
                msg.get_channel_name(),
                result
            );
            notify_tx
                .send(Notification {
                    name: result["name"].as_str().unwrap().to_owned(),
                    contents: result["contents"].as_str().unwrap().to_owned(),
                })
                .expect("Failed to send notification to frontend");
        });

        Ok(RedisBackend { notify: notify_rx })
    }
}

impl VersionInfo for RedisBackend {
    fn version() -> String {
        format!("redis ({})", env!("CARGO_PKG_VERSION"))
    }
}

impl Backend for RedisBackend {
    fn get_notification(&mut self) -> Option<Notification> {
        match self.notify.try_recv() {
            Ok(notification) => Some(notification),
            Err(TryRecvError::Empty) => None,
            Err(e) => panic!("Error: {:?}", e),
        }
    }
}
