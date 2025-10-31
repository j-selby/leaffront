/// Handles OpenGLES textures, and provides mechanisms for interacting/drawing on them
/// safely.
use crate::render::color::Color;

pub struct Texture {
    pub tex_data: Vec<u8>,
    width: usize,
    height: usize,
}

impl Texture {
    pub fn draw_pixel(&mut self, color: &Color, x: usize, y: usize) {
        let starting_pos = (y * self.width + x) * 4;

        self.tex_data[starting_pos] = color.r;
        self.tex_data[starting_pos + 1] = color.g;
        self.tex_data[starting_pos + 2] = color.b;
        self.tex_data[starting_pos + 3] = color.a;
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    /// Creates a new Texture for drawing. This is only uploaded on demand.
    pub fn new(width: usize, height: usize) -> Self {
        Texture {
            tex_data: vec![0; width * height * 4],
            width,
            height,
        }
    }
}
