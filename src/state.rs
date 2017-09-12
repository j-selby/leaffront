/// Represents different states that the display can be in

pub enum ScreenState {
    Day(Message),
    Night
}

impl ScreenState {
    /// Gets the brightness for a particular state.
    pub fn get_brightness(&self) -> u8 {
        // TODO: Use config file
        match self {
            &ScreenState::Day(_) => 100,
            &ScreenState::Night  => 12
        }
    }
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