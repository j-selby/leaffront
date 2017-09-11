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

use pi::gl_context::Context;

use image::load_from_memory;

use std::collections::BTreeMap;

use chrono::Local;

struct TextureCommand {
    pub tex_ptr  : GlTexture,
    pub vertices : Vec<f32>
}

fn gl_loop(context: Context) {
    let dimensions = Context::get_resolution();

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

    let collection = FontCollection::from_bytes(font_data as &[u8]);
    let font = collection.into_font().unwrap(); // only succeeds if collection consists of one font


    let mut counter = fps_counter::FPSCounter::new();

    let mut map = BTreeMap::new();

    let bg_cmd = TextureCommand {
        vertices : [
            -1.0,   1.0,
            -1.0,  -1.0,
            1.0,  -1.0,

            -1.0,  -1.0,
            1.0,   1.0,
            1.0,  -1.0
        ].to_vec(),
        tex_ptr : GlTexture::from_image(&bg_image.to_rgba())
    };

    for _ in 0 .. 5 {
        drawer.start();
        drawer.draw_textured_vertices(&bg_cmd.tex_ptr, &bg_cmd.vertices);

        {
            let time = Local::now();
            let time = time.format("%I:%M:%S %P").to_string();
            let msg = format!("FPS: {}\n{}", counter.tick(), time);
            let layout = font.layout(&msg,
                                     Scale { x: 60.0, y: 50.0 },
                                     point(20.0, 50.0));

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

                // Size the texture for a OpenGL environment
                let min_x = (bounding_box.min.x as f32) / dimensions.width as f32 * 2.0 - 1.0;
                let max_x = (bounding_box.max.x as f32) / dimensions.width as f32 * 2.0 - 1.0;
                let min_y = (bounding_box.min.y as f32) / dimensions.height as f32 * 2.0 - 1.0;
                let max_y = (bounding_box.max.y as f32) / dimensions.height as f32 * 2.0 - 1.0;

                // Generate vertex data
                let vertices = [
                    // Vertex 1
                    min_x, -min_y,
                    min_x, -max_y,
                    max_x, -max_y,
                    // Vertex 2
                    min_x, -max_y,
                    max_x, -min_y,
                    max_x, -max_y,
                ];

                //println!("Vertices: {:?}", vertices);
                //println!("Allocated texture of size: {} {}", tex.get_width(), tex.get_height());

                let tex = map.get(&letter.id()).unwrap();

                // Setup vertice data
                drawer.draw_textured_vertices(tex, &vertices);
            }
        }

        drawer.end();

        std::thread::sleep(std::time::Duration::new(1, 0));
    }
}

fn main() {
    // init egl

    let context = Context::build().unwrap();

    gl_loop(context);
}
