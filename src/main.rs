extern crate egl;
extern crate opengles;
extern crate videocore;

extern crate image;
extern crate rusttype;

extern crate chrono;

extern crate fps_counter;

mod color;
mod texture;

mod gl_render;
mod pi;

use rusttype::{FontCollection, Scale, point};

use color::Color;
use texture::Texture;

use gl_render::texture::GlTexture;
use gl_render::drawer::Drawer;
use gl_render::pos::Position;
use gl_render::pos::Rect;

use pi::gl_context::Context;

use image::load_from_memory;

use std::collections::BTreeMap;

use chrono::Local;

fn gl_loop(context: Context) {
    // init shaders
    let mut drawer = Drawer::new(context);

    // load background image
    println!("Load image:");
    let bg_image = load_from_memory(include_bytes!("../res/bg.jpg")).unwrap();
    println!("Convert");

    println!("Upload");

    println!("font");

    // load rusttype font into memory
    let font_data = include_bytes!("../res/opensans.ttf");

    // TODO: Convert into Font (private)/FontCache
    let collection = FontCollection::from_bytes(font_data as &[u8]);
    let font = collection.into_font().unwrap(); // only succeeds if collection consists of one font


    let mut counter = fps_counter::FPSCounter::new();

    let mut map = BTreeMap::new();

    let bg = GlTexture::from_image(&bg_image.to_rgba());
    // TODO: Manually resize background to correct resolution ourselves

    for _ in 0 .. 5000 {
        drawer.start();
        let screen_width = drawer.get_width() as i32;
        let screen_height = drawer.get_height() as i32;

        drawer.draw_texture_sized(&bg, &Rect::new(0, 0, screen_width, screen_height));

        drawer.enable_blending();

        drawer.draw_colored_rect(&Rect::new(0, screen_height - 100, screen_width, 100),
                                 &Color::new_4byte(0, 0, 0, 100));

        {
            let time = Local::now();
            let time = time.format("%I:%M:%S %P").to_string();
            let msg = format!("FPS: {}\n{}", counter.tick(), time);
            let layout = font.layout(&msg,
                                     Scale { x: 60.0, y: 50.0 },
                                     point(20.0, screen_height as f32 - 50.0));

            let base_color = Color::new_3byte(255, 255, 255);
            for letter in layout {
                // Render out texture
                let bounding_box_opt = letter.pixel_bounding_box();

                if bounding_box_opt.is_none() {
                    continue;
                }

                let bounding_box = bounding_box_opt.unwrap();

                // See if we already have this letter
                if !map.contains_key(&letter.id()) {
                    let mut tex = Texture::new(bounding_box.width() as usize,
                                               bounding_box.height() as usize);

                    {
                        let render_pos = |x: u32, y: u32, factor: f32| {
                            tex.draw_pixel(&base_color.alpha((factor * 255.0) as u8),
                                           x as usize, y as usize);
                        };

                        letter.draw(render_pos);
                    }

                    let opengl_tex = GlTexture::from_texture(&tex);

                    map.insert(letter.id(), opengl_tex);
                }

                //println!("Allocated texture of size: {} {}", tex.get_width(), tex.get_height());

                let tex = map.get(&letter.id()).unwrap();

                // Setup vertice data
                drawer.draw_texture(tex,
                                    &Position::new(bounding_box.min.x as i32,
                                                  bounding_box.min.y as i32));
            }
        }

        drawer.end();

        //std::thread::sleep(std::time::Duration::new(1, 0));
    }
}

fn main() {
    // init egl

    let context = Context::build().unwrap();

    gl_loop(context);
}
