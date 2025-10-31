/// Represents different states that the display can be in
use leaffront_core::backend::Notification;

use std::time::Instant;

#[derive(PartialEq, Eq)]
pub enum ScreenState {
    Day(Message),
    Night,
}

#[derive(PartialEq, Eq)]
pub enum Message {
    Date,
    Weather,
}

impl Message {
    pub fn next(&self) -> Self {
        match self {
            &Message::Date => Message::Weather,
            &Message::Weather => Message::Date,
        }
    }
}

pub struct DisplayNotification {
    pub source: Notification,
    pub displayed: Instant,
}

impl DisplayNotification {
    pub fn new(notify: Notification) -> Self {
        DisplayNotification {
            source: notify,
            displayed: Instant::now(),
        }
    }
}
