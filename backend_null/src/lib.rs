extern crate leaffront_core;

use leaffront_core::backend::Backend;
use leaffront_core::backend::Notification;
use leaffront_core::version::VersionInfo;

pub struct NullBackend {}

impl NullBackend {
    pub fn new() -> Result<Self, ()> {
        Ok(Self {})
    }
}

impl VersionInfo for NullBackend {
    fn version() -> String {
        format!("null ({})", env!("CARGO_PKG_VERSION"))
    }
}

impl Backend for NullBackend {
    fn get_notification(&mut self) -> Option<Notification> {
        None
    }
}
