pub mod color;
pub mod font;
pub mod texture;

use pos::Position;
use pos::Rect;

use render::color::Color;
use render::texture::Texture;

use image::DynamicImage;

/// The dimensions of a object
pub trait Dimensions {
    /// Returns the width of this object.
    fn get_width(&self) -> usize;

    /// Returns the height of this object.
    fn get_height(&self) -> usize;
}

/// Structures for rendering stuff to the screen
pub trait Drawer {
    type NativeTexture: Sized + Dimensions;

    /// Starts a particular rendering frame
    fn start(&mut self);

    /// Ends a frame, requesting for framebuffers to be finalised/etc
    fn end(&mut self);

    /// Clears the frame.
    /// transparent: If the frame should be cleared to alpha 0.
    fn clear(&mut self, transparent: bool);

    /// Enables blending of a texture/etc with the background, if this is
    ///  explicitly required.
    fn enable_blending(&mut self);

    /// Converts a texture to a native reference.
    fn convert_native_texture(&mut self, texture: Texture) -> Self::NativeTexture;

    /// Returns the width of the framebuffer.
    fn get_width(&self) -> usize;

    /// Returns the height of the framebuffer.
    fn get_height(&self) -> usize;

    /// Uses the specified image as a background. This is provided as several platforms
    /// have ways to accelerate this beyond OpenGL calls.
    fn set_background(&mut self, image: DynamicImage);

    /// Sets the screen brightness, if possible. Ignore call if not.
    fn set_brightness(&mut self, brightness: u8) -> ::std::io::Result<()>;

    /// Draws a texture to the screen, with a specified set of vertices to draw to, a UV
    /// to decode the image with, and a color to use as a base.
    fn draw_textured_vertices_colored_uv(
        &mut self,
        texture: &Self::NativeTexture,
        vertices: &[f32],
        colors: &[f32],
        uv: &[f32],
    );

    /// Draws a set of colored vertices to the screen, with a specified color array.
    fn draw_colored_vertices(&mut self, vertices: &[f32], colors: &[f32]);

    /// Returns the count of transitions that occured so far in this frame.
    fn get_transition_count(&self) -> usize;

    /// Draws a texture to the screen, with a specified set of vertices to draw to, and a color
    /// to use as a base.
    fn draw_textured_vertices_colored(
        &mut self,
        texture: &Self::NativeTexture,
        vertices: &[f32],
        colors: &[f32],
    ) {
        self.draw_textured_vertices_colored_uv(
            texture,
            vertices,
            colors,
            &[0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0],
        )
    }

    /// Draws a texture to the screen, with a specified set of vertices to draw to, and a
    /// default UV.
    fn draw_textured_vertices(&mut self, texture: &Self::NativeTexture, vertices: &[f32]) {
        self.draw_textured_vertices_colored(texture, vertices, &[1.0; 24])
    }

    /// Draws a texture to the screen, with the specified x/y coordinates (relative to screen size),
    ///  and a specified width/height.
    fn draw_texture_sized(&mut self, texture: &Self::NativeTexture, rect: &Rect, color: &Color) {
        let vertices = self.rect_to_vertices(rect);

        let mut colors: [f32; 24] = [0.0; 24];

        for i in 0..24 / 4 {
            colors[i * 4] = f32::from(color.r) / 255.0;
            colors[i * 4 + 1] = f32::from(color.g) / 255.0;
            colors[i * 4 + 2] = f32::from(color.b) / 255.0;
            colors[i * 4 + 3] = f32::from(color.a) / 255.0;
        }

        self.draw_textured_vertices_colored(texture, &vertices, &colors)
    }

    /// Draws a texture to the screen, with the specified x/y coordinates (relative to screen size),
    /// and the texture dimensions as width/height.
    fn draw_texture_colored(
        &mut self,
        texture: &Self::NativeTexture,
        pos: &Position,
        color: &Color,
    ) {
        let width = texture.get_width();
        let height = texture.get_height();

        self.draw_texture_sized(
            texture,
            &Rect::new_from_pos(pos, width as i32, height as i32),
            color,
        )
    }

    /// Draws a texture to the screen, with the specified x/y coordinates (relative to screen size),
    /// and the texture dimensions as width/height.
    fn draw_texture(&mut self, texture: &Self::NativeTexture, pos: &Position) {
        // TODO: Potentially dedicated shader for non colored?
        self.draw_texture_colored(texture, pos, &Color::new_4byte(255, 255, 255, 255))
    }

    /// Draws a colored rectangle to the screen, with a single color.
    fn draw_colored_rect(&mut self, rect: &Rect, color: &Color) {
        let vertices: [f32; 12] = self.rect_to_vertices(&rect);
        let mut colors: [f32; 24] = [0.0; 24];

        for i in 0..24 / 4 {
            colors[i * 4] = f32::from(color.r) / 255.0;
            colors[i * 4 + 1] = f32::from(color.g) / 255.0;
            colors[i * 4 + 2] = f32::from(color.b) / 255.0;
            colors[i * 4 + 3] = f32::from(color.a) / 255.0;
        }

        self.draw_colored_vertices(&vertices, &colors)
    }

    /// Converts a rectangle to a minimum and maximum bounding point
    fn rect_to_min_max(&self, rect: &Rect) -> ((f32, f32), (f32, f32)) {
        let min_x = (rect.x as f32) / self.get_width() as f32 * 2.0 - 1.0;
        let max_x = ((rect.x + rect.width) as f32) / self.get_width() as f32 * 2.0 - 1.0;
        let min_y = (rect.y as f32) / self.get_height() as f32 * 2.0 - 1.0;
        let max_y = ((rect.y + rect.height) as f32) / self.get_height() as f32 * 2.0 - 1.0;

        ((min_x, min_y), (max_x, max_y))
    }
    /// Converts a rectangle to 4 vertices
    fn rect_to_vertices(&self, rect: &Rect) -> [f32; 12] {
        // Translate to OpenGL coordinates
        let ((min_x, min_y), (max_x, max_y)) = self.rect_to_min_max(rect);

        // Generate vertex data
        // Inverted due to OpenGL perspective
        [
            // Vertex 1
            min_x, -min_y, min_x, -max_y, max_x, -max_y, // Vertex 2
            min_x, -min_y, max_x, -min_y, max_x, -max_y,
        ]
    }

    /// Clips the current render to the specified location
    fn start_clip(&self, rect: &Rect);

    /// Finishes any clipping operation
    fn end_clip(&self);
}
