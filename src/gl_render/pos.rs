/// Represents a X/Y position.

pub struct Position {
    pub x : i32,
    pub y : i32
}

impl Position {
    pub fn new(x : i32, y : i32) -> Self {
        Position {
            x, y
        }
    }
}
