/// Allows for the caching of fonts.

use rusttype::Font;
use rusttype::GlyphId;
use rusttype::FontCollection;
use rusttype::Scale;
use rusttype::Point;

use gl_render::texture::GlTexture;
use gl_render::drawer::Drawer;
use gl_render::pos::Position;

use color::Color;

use texture::Texture;

use std::collections::BTreeMap;

#[derive(Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
struct CachedGlyph {
    id    : GlyphId,
    color : Color,
    size  : i32
}

pub struct FontCache<'a> {
    font  : Font<'a>,
    cache : BTreeMap<CachedGlyph, GlTexture>
}

impl<'a> FontCache<'a> {
    /// Draws the specified string to the screen.
    pub fn draw(&mut self, text : &str, color : &Color, size : i32, pos : &Position,
                draw : &mut Drawer) {
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

                let opengl_tex = GlTexture::from_texture(&tex);

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
