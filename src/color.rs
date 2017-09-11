/// Represents a unsigned OpenGL color in Rust form.

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Color {
    pub r : u8,
    pub g : u8,
    pub b : u8,
    pub a : u8
}

impl Color {
    pub fn alpha(&self, a : u8) -> Self {
        let mut cloned = self.clone();
        cloned.a = a;
        cloned
    }

    pub fn new_4byte(r : u8, g : u8, b : u8, a : u8) -> Self {
        Color {
            r, g, b, a
        }
    }

    pub fn new_3byte(r : u8, g : u8, b : u8) -> Self {
        Color::new_4byte(r, g, b, 255)
    }
}
