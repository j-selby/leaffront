/// Allows for the caching of fonts.

use rusttype::Font;
use rusttype::GlyphId;
use rusttype::FontCollection;
use rusttype::Scale;
use rusttype::Point;

use pos::Position;

use render::Drawer;
use render::Dimensions;
use render::color::Color;
use render::texture::Texture;

use std::collections::BTreeMap;

#[derive(Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
struct CachedGlyph {
    id    : GlyphId,
    color : Color,
    size  : i32
}

pub struct FontCache<'a, T : Sized> {
    font  : Font<'a>,
    cache : BTreeMap<CachedGlyph, T>
}

impl<'a, T : Dimensions> FontCache<'a, T> {
    /// Returns the width of a specified string.
    pub fn get_width(&self, text : &str, size : i32) -> i32 {
        let layout = self.font.layout(text, Scale::uniform(size as f32),
                                      Point { x : 0.0, y : 0.0 });

        let mut width = 0;
        for char in layout {
            let bb = char.pixel_bounding_box();
            match bb {
                Some(bb) => {
                    let pos = char.position().x as i32 + bb.width();
                    if pos > width {
                        width = pos;
                    }
                },
                None => {},
            }
        }

        width
    }

    /// Draws the specified string to the screen.
    pub fn draw(&mut self, text : &str, color : &Color, size : i32, pos : &Position,
                draw : &mut Drawer<NativeTexture=T>) {
        let layout = self.font.layout(text, Scale::uniform(size as f32),
                                      Point { x : pos.x as f32, y : pos.y as f32 });

        for glyph in layout {
            // Render out texture
            let bounding_box_opt = glyph.pixel_bounding_box();

            if bounding_box_opt.is_none() {
                continue;
            }

            let bounding_box = bounding_box_opt.unwrap();

            // Build hash ID for this glyph
            let id = CachedGlyph {
                id : glyph.id(),
                color : color.clone(),
                size
            };

            if !self.cache.contains_key(&id) {
                let mut tex = Texture::new(bounding_box.width() as usize,
                                           bounding_box.height() as usize);

                {
                    let render_pos = |x: u32, y: u32, factor: f32| {
                        tex.draw_pixel(&color.alpha((factor * 255.0) as u8),
                                       x as usize, y as usize);
                    };

                    glyph.draw(render_pos);
                }

                let opengl_tex = draw.convert_native_texture(tex);

                self.cache.insert(id.clone(), opengl_tex);
            }

            // TODO: Layout text vertically
            let tex = self.cache.get(&id).unwrap();

            // Setup vertice data
            draw.draw_texture(tex,
                                &Position::new(bounding_box.min.x as i32,
                                               bounding_box.min.y as i32));
        }

    }

    /// Creates a new cache from a .ttf file.
    pub fn from_bytes(data : &'a [u8]) -> Self {
        let collection = FontCollection::from_bytes(data);

        // only succeeds if collection consists of one font
        let font = collection.into_font().unwrap();

        FontCache {
            font,
            cache : BTreeMap::new()
        }
    }
}
