extern crate egl;
extern crate opengles;
extern crate videocore;

extern crate image;
extern crate rusttype;

extern crate chrono;

extern crate fps_counter;

extern crate libc;

mod color;
mod texture;

mod gl_render;
mod pi;

use color::Color;
use texture::Texture;

use gl_render::texture::GlTexture;
use gl_render::drawer::Drawer;
use gl_render::pos::Position;
use gl_render::pos::Rect;
use gl_render::font::FontCache;

use pi::gl_context::Context;

use image::load_from_memory;

use chrono::Local;
use chrono::Datelike;

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
    //let collection = FontCollection::from_bytes(font_data as &[u8]);
    //let font = collection.into_font().unwrap(); // only succeeds if collection consists of one font


    let mut counter = fps_counter::FPSCounter::new();

    let mut font = FontCache::from_bytes(font_data);

    let bg = GlTexture::from_image(&bg_image.to_rgba());
    // TODO: Manually resize background to correct resolution ourselves

    loop {
        drawer.start();
        let screen_width = drawer.get_width() as i32;
        let screen_height = drawer.get_height() as i32;

        drawer.draw_texture_sized(&bg, &Rect::new(0, 0, screen_width, screen_height));

        drawer.enable_blending();

        drawer.draw_colored_rect(&Rect::new(0, screen_height - 120, screen_width, 120),
                                 &Color::new_4byte(0, 0, 0, 100));

        let datetime = Local::now();
        let time = datetime.format("%-I:%M:%S %P").to_string();
        let msg = format!("{}", time);

        font.draw(&msg,  &Color::new_3byte(255, 255, 255),
                  50, &Position::new(20, screen_height - 75), &mut drawer);

        let suffix = match datetime.day() {
            1 | 21 | 31 => "st",
            2 | 22 => "nd",
            3 | 23 => "rd",
            _ => "th",
        };

        //let time = datetime.format("%A, %-D").to_string();
        let msg = format!("{}{} of {}", datetime.format("%A, %-d").to_string(), suffix,
                          datetime.format("%B").to_string());

        font.draw(&msg,  &Color::new_3byte(255, 255, 255),
                  50, &Position::new(20, screen_height - 25), &mut drawer);

        font.draw(&format!("FPS: {}", counter.tick()),
                  &Color::new_3byte(255, 255, 255),
                  20, &Position::new(20, 50), &mut drawer);

        drawer.end();
        drawer.vsync();
    }
}

fn main() {
    // init egl

    let context = Context::build().unwrap();

    gl_loop(context);
}
