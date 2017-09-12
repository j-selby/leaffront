extern crate egl;
extern crate opengles;
extern crate videocore;

extern crate image;
extern crate rusttype;

extern crate chrono;
extern crate ftp;
extern crate xmltree;
extern crate rand;

extern crate fps_counter;

extern crate libc;

mod color;
mod texture;
mod state;

mod gl_render;
mod pi;
mod weather;

use color::Color;
use state::ScreenState;
use state::Message;
use weather::manager::Manager;

use gl_render::texture::GlTexture;
use gl_render::drawer::Drawer;
use gl_render::pos::Position;
use gl_render::pos::Rect;
use gl_render::font::FontCache;

use videocore::dispmanx;
use videocore::image::ImageType;
use videocore::image::Rect as VCRect;

use pi::gl_context::Context;
use pi::brightness::set_brightness;

use image::load_from_memory;
use image::GenericImage;

use chrono::Local;
use chrono::Datelike;

use rand::Rng;
use rand::thread_rng;

fn gl_loop(context: Context) {
    // Create our mechanism for rendering
    let mut drawer = Drawer::new(context);

    println!("Drawer go!");

    // TODO: Check startup time
    let mut state = ScreenState::Night;//ScreenState::Day(Message::Date);
    set_brightness(state.get_brightness()).unwrap();

    println!("Brightness go!");
    let mut state_countdown : u32 = 0;

    let bg_image = load_from_memory(include_bytes!("../res/bg.jpg")).unwrap();

    let font_data = include_bytes!("../res/opensans.ttf");

    let mut counter = fps_counter::FPSCounter::new();

    let mut font = FontCache::from_bytes(font_data);

    // Enable on non-videocore platforms
    //let bg = GlTexture::from_image(&bg_image.to_rgba());
    // TODO: Manually resize background to correct resolution ourselves

    let mut weather_manager = Manager::new(20 * 60 * 1000);

    let mut rng = thread_rng();
    let mut night_x = -1;
    let mut night_y = -1;

    // Update the background
    let mut dest_rect = VCRect {
        x: 0,
        y: 0,
        width: drawer.get_width() as i32,
        height: drawer.get_width() as i32
    };

    let mut bg_img = bg_image.to_rgb();
    let bg_ptr = bg_img.as_mut_ptr() as *mut _ as *mut libc::c_void;

    println!("Resources write go!");
    if dispmanx::resource_write_data(drawer.context.bg_resource, ImageType::RGB888,
                                  (3 * bg_image.width()) as i32,
                                  bg_ptr, &dest_rect) {
        println!("Failed to write data")
    }

    println!("Resources modified go!");
    if dispmanx::element_modified(drawer.context.update, drawer.context.bg_element, &mut dest_rect) {
        println!("Failed to notify update");
    }

    println!("Resources update go!");
    if dispmanx::update_submit_sync(drawer.context.update) {
        println!("Failed to update");
    }

    println!("Resources go!");
    loop {
        let next_state = match &state {
            &ScreenState::Day(ref msg) => {
                state_countdown += 1;
                if state_countdown > 60 * 5 {
                    state_countdown = 0;
                    Some(ScreenState::Day(msg.next()))
                } else {
                    None
                }
            },
            &ScreenState::Night => {
                None
            }
        };

        match next_state {
            Some(next) => {
                state = next;
                set_brightness(state.get_brightness()).unwrap();
            }
            None => {}
        }

        drawer.start();
        let screen_width = drawer.get_width() as i32;
        let screen_height = drawer.get_height() as i32;

        match &state {
            &ScreenState::Day(ref subtitle) => {
                drawer.clear(true);
                // Enable on non-Videocore platforms
                /*drawer.draw_texture_sized(&bg, &Rect::new(0, 0, screen_width, screen_height),
                                          &Color::new_4byte(255, 255, 255, 255));*/

                drawer.enable_blending();

                drawer.draw_colored_rect(&Rect::new(0, screen_height - 120, screen_width, 120),
                                         &Color::new_4byte(0, 0, 0, 100));

                let datetime = Local::now();
                let time = datetime.format("%-I:%M:%S %P").to_string();
                let msg = format!("{}", time);

                font.draw(&msg,  &Color::new_3byte(255, 255, 255),
                          50, &Position::new(20, screen_height - 75), &mut drawer);

                match subtitle {
                    &Message::Date => {
                        let suffix = match datetime.day() {
                            1 | 21 | 31 => "st",
                            2 | 22 => "nd",
                            3 | 23 => "rd",
                            _ => "th",
                        };

                        let msg = format!("{}{} of {}", datetime.format("%A, %-d").to_string(), suffix,
                                          datetime.format("%B").to_string());

                        font.draw(&msg,  &Color::new_3byte(255, 255, 255),
                                  50, &Position::new(20, screen_height - 25), &mut drawer);
                    },
                    &Message::Weather => {
                        let weather = weather_manager.get();
                        let msg =
                            match weather {
                                Ok(weather) => format!("{}°C - {}", weather.temperature, weather.description),
                                Err(msg) => msg
                            };

                        font.draw(&msg,  &Color::new_3byte(255, 255, 255),
                                  50, &Position::new(20, screen_height - 25), &mut drawer);
                    }
                }

                font.draw(&format!("FPS: {}", counter.tick()),
                          &Color::new_3byte(255, 255, 255),
                          20, &Position::new(20, 50), &mut drawer);
            },
            &ScreenState::Night => {
                drawer.clear(false);

                // Render out both the top and bottom strings, and center them.
                let datetime = Local::now();
                let top_msg = datetime.format("%-I:%M:%S %P").to_string();
                let top_length = font.get_width(&top_msg, 50);
                let top_two = top_length / 2;

                let suffix = match datetime.day() {
                    1 | 21 | 31 => "st",
                    2 | 22 => "nd",
                    3 | 23 => "rd",
                    _ => "th",
                };

                let bottom_msg = format!("{}{} of {}", datetime.format("%A, %-d").to_string(), suffix,
                                  datetime.format("%B").to_string());
                let bottom_length = font.get_width(&bottom_msg, 50);
                let bottom_two = bottom_length / 2;

                state_countdown += 1;
                if state_countdown > 60 * 5 || night_x == -1 {
                    state_countdown = 0;

                    // Set new random position
                    // Calculate maximum ranges
                    let max_width = if top_two > bottom_two { top_two } else { bottom_two };
                    let max_x = screen_width - max_width;
                    let min_x = max_width;

                    let min_y = 50; // For font
                    let max_y = screen_height - 50 * 3; // For font + gap

                    night_x = rng.gen_range(min_x, max_x);
                    night_y = rng.gen_range(min_y, max_y);

                }

                let top_x = night_x - top_two;
                let bottom_x = night_x - bottom_two;

                drawer.enable_blending();

                font.draw(&top_msg,  &Color::new_3byte(255, 255, 255),
                          50, &Position::new(top_x, night_y), &mut drawer);
                font.draw(&bottom_msg,  &Color::new_3byte(255, 255, 255),
                          50, &Position::new(bottom_x, night_y + 50), &mut drawer);
            }
        }



        drawer.end();
        //drawer.vsync();
    }
}

fn main() {
    // init egl

    let context = Context::build().unwrap();

    gl_loop(context);
}

/*
extern crate libc;
extern crate egl;
extern crate opengles;
extern crate videocore;

extern crate image;
extern crate rusttype;
extern crate chrono;
extern crate ftp;
extern crate xmltree;

extern crate fps_counter;

//mod binds;

mod color;
mod texture;

mod gl_render;
mod pi;
mod weather;

use color::Color;
use texture::Texture;

use videocore::dispmanx;
use videocore::dispmanx::{FlagsAlpha, Transform, VCAlpha, Window, DisplayHandle, UpdateHandle,
                          ElementHandle, ResourceHandle};
use videocore::image::Rect as VCRect;
use videocore::image::ImageType;
use videocore::bcm_host;

use weather::manager::Manager;
use weather::Weather;

use std::mem::transmute;

use gl_render::texture::GlTexture;
//use gl_render::drawer::Drawer;
use gl_render::pos::Position;
use gl_render::pos::Rect;
//use gl_render::font::FontCache;

//use pi::gl_context::Context;

use image::load_from_memory;

use chrono::Local;
use chrono::Datelike;

use std::time::Instant;

use std::ptr;

fn gl_loop() {//context: Context) {
    // init shaders


    // Update weather every 20 minutes
    //let mut weather = Manager::new(20 * 60 * 1000);

    // load background image
    //println!("Load image:");
    //let bg2_image = load_from_memory(include_bytes!("../res/bg2.png")).unwrap();
    //println!("Convert");

    //println!("Upload");

    //println!("font");

    // load rusttype font into memory
    //let font_data = include_bytes!("../res/opensans.ttf");

    // TODO: Convert into Font (private)/FontCache
    //let collection = FontCollection::from_bytes(font_data as &[u8]);
    //let font = collection.into_font().unwrap(); // only succeeds if collection consists of one font


    //let mut counter = fps_counter::FPSCounter::new();

   //let mut font = FontCache::from_bytes(font_data);

    //let mut bg = GlTexture::from_image(&bg_image.to_rgba());
    //let mut bg2 = GlTexture::from_image(&bg2_image.to_rgba());

    //let mut drawer = Drawer::new(context);

    // image crate provides load_from_memory
    let bg_image = load_from_memory(include_bytes!("../res/bg2.png")).unwrap();

    bcm_host::init();

    // open the display
    let display = dispmanx::display_open(0);

    let size = bcm_host::graphics_get_display_size(0).unwrap();

    // Upload bg to the videocore
    let mut dest_rect = VCRect {
        x: 0,
        y: 0,
        width: size.width as i32,
        height: size.height as i32
    };

    let mut src_rect = VCRect {
        x: 0,
        y: 0,
        width: size.width as i32,
        height: size.height as i32
    };

    let mut bg_img = bg_image.to_rgb();

    let mut ptr = 0; // Not used

    let update = dispmanx::update_start(10);
    assert!(update != 0);

    let mut req: [libc::uint32_t; 1] = [ImageType::RGB888 as libc::uint32_t];
    println!("Supported: {}", dispmanx::query_image_formats(req.as_mut_ptr()));

    let mut data : Vec<u8> = vec![255; (size.width as usize * 3) * size.height as usize];

    let bg_ptr = bg_img.as_mut_ptr() as *mut _ as *mut libc::c_void;

    println!("Create resource:");
    let bg_resource = dispmanx::resource_create(ImageType::RGB888,
        size.width as u32, size.height as u32,
                                                &mut ptr);
    assert!(bg_resource != 0);

    //let bg_resource = unsafe { binds::gen_resource(display, update, bg_resource) };

    println!("Upload resource data:");
    let mut dest_rect =VCRect {
        x: 0,
        y: 0,
        width: size.width as i32,
        height: size.height as i32
    } /*binds::tag_VC_RECT_T {
        x : 0,
        y : 0,
        width : size.width as i32,
        height : size.height as i32
    };*/;

    let result = unsafe {
        /*binds::vc_dispmanx_resource_write_data(bg_resource,
                                               binds::VC_IMAGE_TYPE_T::VC_IMAGE_RGB888,
                                               (3 * size.width) as i32,
                                               data.as_mut_ptr() as *mut _ as *mut ::std::os::raw::c_void, &mut dest_rect)*/
        dispmanx::resource_write_data(bg_resource, ImageType::RGB888,
                                      (3 * size.width) as i32,
                                      bg_ptr, &dest_rect)
        //binds::gen_resource(bg_resource)

    };

    if result {
        println!("Failed to update resource!");
    }

    /*let bg_resource = unsafe { binds::gen_resource(display) };*/

    let flag1: u32 = unsafe { ::std::mem::transmute(FlagsAlpha::FROM_SOURCE) };
    let flag2: u32 = unsafe { ::std::mem::transmute(FlagsAlpha::FIXED_ALL_PIXELS) };

    let mut alpha = VCAlpha {
        flags: unsafe { ::std::mem::transmute(flag1 | flag2) },
        opacity: 120,
        mask: 0
    };

    let mut dest_rect2 = VCRect {
        x: 0,
        y: 0,
        width: (size.width as i32),
        height: (size.height as i32)
    };

    let mut src_rect2 = VCRect {
        x: 0,
        y: 0,
        width: (size.width as i32) << 16,
        height: (size.height as i32) << 16
    };

    println!("Upload element:");
    let bg_element = /*unsafe { binds::gen_resource2(display, update, bg_resource) };*/dispmanx::element_add(update, display,
                                           3, &mut dest_rect2,
                                           bg_resource, &mut src_rect2,
                                           dispmanx::DISPMANX_PROTECTION_NONE,
                                           &mut alpha,
                                           ptr::null_mut(),
                                           Transform::NO_ROTATE);
    //let bg_element = unsafe { binds::gen_resource(display, update,  bg_resource, &mut dest_rect2, &mut src_rect2) };
    assert!(bg_element != 0);

    //dispmanx::element_modified(update, bg_element, &mut dest_rect);
    if dispmanx::update_submit_sync(update) {
        println!("Failed to update!");
    }

    println!("Begin wait.");
    loop {}

    // TODO: Move these to dispman?
    // TODO: Manually resize background to correct resolution ourselves

    // TODO: Convert to hashmap, add callbacks with context
    /*let wakeups : Vec<Instant> = Vec::new();

    let mut count : u8 = 0;
    let mut switcher : u16 = 0;
    loop {
        drawer.start();
        let screen_width = drawer.get_width() as i32;
        let screen_height = drawer.get_height() as i32;

        count = count.wrapping_add(1);
        /*if count == 0 {
            let copy = bg2;
            bg2 = bg;
            bg = copy;
        }*/

        switcher += 1;
        if switcher > 60 * 10 {
            switcher = 0;
        }

        //drawer.draw_texture_sized(&bg, &Rect::new(0, 0, screen_width, screen_height),
        //                          &Color::new_4byte(255, 255, 255, 255));

        drawer.enable_blending();

        //drawer.draw_texture_sized(&bg2, &Rect::new(0, 0, screen_width, screen_height),
        //                          &Color::new_4byte(255, 255, 255, count));

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
        let msg =
            if switcher <= 60 * 5 {
                format!("{}{} of {}", datetime.format("%A, %-d").to_string(), suffix,
                        datetime.format("%B").to_string())
            } else {
                let weather = weather.get();
                match weather {
                    Ok(weather) => format!("{}°C - {}", weather.temperature, weather.description),
                    Err(msg) => msg,
                }
            };

        font.draw(&msg,  &Color::new_3byte(255, 255, 255),
                  50, &Position::new(20, screen_height - 25), &mut drawer);

        font.draw(&format!("FPS: {}", counter.tick()),
                  &Color::new_3byte(255, 255, 255),
                  20, &Position::new(20, 50), &mut drawer);

        drawer.end();
        drawer.vsync();
    }*/
}

fn main() {
    // init egl

    //let context = Context::build().unwrap();

    gl_loop();//context);
}
*/