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
use texture::Texture;
use state::ScreenState;
use state::Message;
use weather::manager::Manager;

use gl_render::texture::GlTexture;
use gl_render::drawer::Drawer;
use gl_render::pos::Position;
use gl_render::pos::Rect;
use gl_render::font::FontCache;

use pi::gl_context::Context;
use pi::brightness::set_brightness;

use image::load_from_memory;

use chrono::Local;
use chrono::Datelike;

use rand::Rng;
use rand::thread_rng;

fn gl_loop(context: Context) {
    // Create our mechanism for rendering
    let mut drawer = Drawer::new(context);

    // TODO: Check startup time
    let mut state = ScreenState::Night;//ScreenState::Day(Message::Date);
    set_brightness(state.get_brightness()).unwrap();

    let mut state_countdown : u32 = 0;

    let bg_image = load_from_memory(include_bytes!("../res/bg.jpg")).unwrap();

    let font_data = include_bytes!("../res/opensans.ttf");

    let mut counter = fps_counter::FPSCounter::new();

    let mut font = FontCache::from_bytes(font_data);

    let bg = GlTexture::from_image(&bg_image.to_rgba());
    // TODO: Manually resize background to correct resolution ourselves

    let mut weather_manager = Manager::new(20 * 60 * 1000);

    let mut rng = thread_rng();
    let mut night_x = -1;
    let mut night_y = -1;

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
                drawer.draw_texture_sized(&bg, &Rect::new(0, 0, screen_width, screen_height),
                                          &Color::new_4byte(255, 255, 255, 255));

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
