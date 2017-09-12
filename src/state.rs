/// Represents different states that the display can be in

pub enum ScreenState {
    Day(Message),
    Night
}

pub enum Message {
    Date,
    Weather
}

impl Message {
    pub fn next(&self) -> Self {
        match self {
            &Message::Date => Message::Weather,
            &Message::Weather => Message::Date
        }
    }
}