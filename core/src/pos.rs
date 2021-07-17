/// Represents a X/Y position.
#[derive(Copy, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Position { x, y }
    }
}

/// Represents a screen-space X/Y position, width and height.
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new_from_pos(pos: &Position, width: i32, height: i32) -> Self {
        Rect {
            x: pos.x,
            y: pos.y,
            width,
            height,
        }
    }

    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    pub fn new_from_logical_space(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        screen_dims: &(usize, usize),
    ) -> Self {
        Rect {
            x: (x * (screen_dims.0 as f32)) as _,
            y: (y * (screen_dims.1 as f32)) as _,
            width: (width * (screen_dims.0 as f32)) as _,
            height: (height * (screen_dims.1 as f32)) as _,
        }
    }
}
