/// Allows for the caching of fonts.
use rusttype::Font;
use rusttype::GlyphId;
use rusttype::Point;
use rusttype::Scale;

use pos::Position;

use render::color::Color;
use render::texture::Texture;
use render::Dimensions;
use render::Drawer;

use std::collections::BTreeMap;
use std::cmp::max;

#[derive(Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
struct CachedGlyph {
    id: GlyphId,
    color: Color,
    size: i32,
}

pub struct FontCache<'a, T: Sized> {
    font: Font<'a>,
    cache: BTreeMap<CachedGlyph, T>,
}

impl<'a, T: Dimensions> FontCache<'a, T> {
    /// Returns the width of a specified string.
    pub fn get_width(&self, text: &str, size: i32) -> i32 {
        let layout = self
            .font
            .layout(text, Scale::uniform(size as f32), Point { x: 0.0, y: 0.0 });

        let mut width = 0;
        for char in layout {
            let bb = char.pixel_bounding_box();
            if let Some(bb) = bb {
                let pos = char.position().x as i32 + bb.width();
                if pos > width {
                    width = pos;
                }
            }
        }

        width
    }

    /// Draws the specified single-line string to the screen.
    pub fn draw<DrawInstance: Drawer<NativeTexture = T>>(
        &mut self,
        text: &str,
        color: &Color,
        size: i32,
        pos: &Position,
        draw: &mut DrawInstance,
    ) {
        let metrics = self.font.v_metrics(Scale::uniform(size as f32));

        let layout = self.font.layout(
            text,
            Scale::uniform(size as f32),
            Point {
                x: pos.x as f32,
                y: pos.y as f32 + metrics.ascent,
            },
        );

        for glyph in layout {
            // Render out texture
            let bounding_box_opt = glyph.pixel_bounding_box();

            if bounding_box_opt.is_none() {
                continue;
            }

            let bounding_box = bounding_box_opt.unwrap();

            // Build hash ID for this glyph
            let id = CachedGlyph {
                id: glyph.id(),
                color: color.clone(),
                size,
            };

            if !self.cache.contains_key(&id) {
                let mut tex = Texture::new(
                    bounding_box.width() as usize,
                    bounding_box.height() as usize,
                );

                {
                    let render_pos = |x: u32, y: u32, factor: f32| {
                        tex.draw_pixel(
                            &color.alpha((factor * 255.0) as u8),
                            x as usize,
                            y as usize,
                        );
                    };

                    glyph.draw(render_pos);
                }

                let opengl_tex = draw.convert_native_texture(tex);

                self.cache.insert(id.clone(), opengl_tex);
            }

            let tex = &self.cache[&id];

            // Setup vertice data
            draw.draw_texture(
                tex,
                &Position::new(bounding_box.min.x as i32, bounding_box.min.y as i32),
            );
        }
    }

    /// Draws multiple lines of text, bounded by a bounding box.
    ///
    /// Returns the physical space used by the line drawer.
    pub fn draw_lines<DrawInstance: Drawer<NativeTexture = T>>(
        &mut self,
        text: &str,
        color: &Color,
        size: i32,
        pos: &Position,
        bounding_box: (Option<i32>, Option<i32>),
        draw: &mut DrawInstance,
    ) -> (i32, i32) {
        let metrics = self.font.v_metrics(Scale::uniform(size as f32));

        // Layout the text, breaking on spaces and after dashes
        let (bounding_x, bounding_y) = bounding_box;

        let bounding_x = match bounding_x {
            Some(x) => x,
            None => {
                // Not enough info - just draw and move on.
                self.draw(text, color, size, pos, draw);

                // Just a single line - return the single line metrics.
                return (self.get_width(text, size), metrics.ascent as _);
            }
        };

        let mut current_string = text;
        // Where to draw the next time
        let mut modified_pos = pos.to_owned();

        // The bounded extent of the lines
        let mut max_x = 0;
        let mut max_y = 0;

        'string_loop:
        while current_string.len() > 0 {
            let mut last_break_point = 0;
            let mut last_width = 0;

            let mut string_to_render = current_string;

            // Make sure we haven't hit the end of our vertical space
            if let Some(y) = bounding_y {
                if modified_pos.y >= pos.y + y {
                    // Out of vertical space
                    println!("Out of vertical space! ({} >= {}), x bounding: {}", pos.y, y, bounding_x);
                    break 'string_loop;
                }
            }

            // Iterate through characters to find the point to end
            'character_loop:
            for (i, character) in current_string.char_indices() {
                let (cut_string, _) = current_string.split_at(i);
                let current_width = self.get_width(cut_string, size);

                if character == ' ' {
                    last_break_point = i + 1;
                    last_width = current_width;
                }

                if current_width > bounding_x {
                    // Time to break the line
                    if last_break_point == 0 {
                        // Have to cut within the word
                        last_break_point = i;
                    }

                    let (new_string_to_render, leftovers) = current_string.split_at(last_break_point);
                    string_to_render = new_string_to_render;
                    current_string = leftovers;

                    break 'character_loop;
                }
            }

            // Draw this individual line
            self.draw(string_to_render, color, size, &modified_pos, draw);

            // Move our position to the next line
            modified_pos.y += metrics.line_gap as i32;

            max_x = max(max_x, last_width);
            max_y = modified_pos.y;

            // Check to see if we have drawn everything left
            if string_to_render == current_string {
                break 'string_loop;
            }
        }

        (max_x, max_y)
    }

    /// Creates a new cache from a .ttf file.
    pub fn from_bytes(data: &'a [u8]) -> Self {
        let font = Font::try_from_bytes(data).expect("Failed to read font");

        FontCache {
            font,
            cache: BTreeMap::new(),
        }
    }
}
