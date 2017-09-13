extern crate egl;
extern crate opengles;
extern crate videocore;
extern crate evdev;

extern crate image;
extern crate rusttype;

extern crate chrono;
extern crate ftp;
extern crate xmltree;
extern crate rand;

extern crate libc;
extern crate ctrlc;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod color;
mod texture;
mod state;

mod gl_render;
mod pi;
mod weather;
mod background;

use color::Color;
use state::ScreenState;
use state::Message;
use weather::manager::WeatherManager;
use background::manager::BackgroundManager;

use gl_render::drawer::Drawer;
use gl_render::pos::Position;
use gl_render::pos::Rect;
use gl_render::font::FontCache;

use pi::gl_context::Context;
use pi::brightness::set_brightness;

use chrono::Local;
use chrono::Datelike;
use chrono::naive::NaiveTime;
use chrono::Duration as cDuration;

use rand::Rng;
use rand::thread_rng;

use std::thread;
use std::time::Duration;
use std::time::Instant;

pub static VERSION : &'static str = "2.0.0";

fn check_night(start_night : u32, end_night : u32) -> bool {
    let time = Local::now();
    let start_time = NaiveTime::from_hms(start_night, 0, 0);
    let end_time = NaiveTime::from_hms(end_night, 0, 0);

    let start_date = time.date().naive_local().and_time(start_time);

    let end_date = if start_time > end_time {
        // End night is on the next day
        let end_date = time.date().naive_local();
        let end_date = end_date + cDuration::days(1);
        end_date.and_time(end_time)
    } else {
        time.date().naive_local().and_time(end_time)
    };

    let cur_date = time.naive_local();

    cur_date > start_date && cur_date < end_date
}

fn gl_loop(context: Context) {
    // TODO: Move to config file
    let start_night = 22;
    let end_night = 7;

    // Create our mechanism for rendering
    let mut drawer = Drawer::new(context);

    // Startup input handling
    let mut devices = evdev::enumerate();

    for device in &devices {
        println!("Found input device: {:?}", device.name());
    }

    let mut check_input = || -> Vec<evdev::raw::input_event> {
        let mut input = Vec::new();
        for mut device  in &mut devices {
            for evt in  device.events_no_sync().unwrap() {
                input.push(evt);
            }
        }
        input
    };

    // Check the startup time
    let mut state = if check_night(start_night, end_night) {
        ScreenState::Night
    } else {
        ScreenState::Day(Message::Date)
    };

    set_brightness(state.get_brightness()).unwrap();

    let mut state_countdown = Instant::now();

    let font_data = include_bytes!("../res/opensans.ttf");

    let mut font = FontCache::from_bytes(font_data);

    // Enable on non-videocore platforms
    //let bg = GlTexture::from_image(&bg_image.to_rgba());

    let mut weather_manager = WeatherManager::new(20 * 60 * 1000);

    let mut rng = thread_rng();
    let mut night_x = -1;
    let mut night_y = -1;
    let mut night_cooldown = Instant::now();

    // Update the background
    let bg_mgr = BackgroundManager::new("art".into(), drawer.context.bg_element);
    let mut bg_countdown = Instant::now();
    bg_mgr.next();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    let has_called = Arc::new(AtomicBool::new(false));

    ctrlc::set_handler(move || {
        println!("Ctrl-C received");
        if has_called.load(Ordering::SeqCst) {
            println!("Forcing shutdown:");
            ::std::process::exit(1);
        } else {
            has_called.store(true, Ordering::SeqCst);
            r.store(false, Ordering::SeqCst);
        }
    }).expect("Error setting Ctrl-C handler");

    println!("Initialised successfully.");

    // TODO: Mechanism to immediately wake up loop
    while running.load(Ordering::SeqCst) {
        let input = check_input();

        let mut touched = false;

        for input in input {
            if input._type == 3 {
                touched = true;
                break;
            }
        }

        let next_state = match &state {
            &ScreenState::Day(ref msg) => {
                // TODO: Move time lengths into config
                if bg_countdown.elapsed() > Duration::from_secs(10) {
                    bg_countdown = Instant::now();
                    bg_mgr.next();
                }

                if night_cooldown.elapsed() > Duration::from_secs(10) &&
                    check_night(start_night, end_night) {
                    state_countdown = Instant::now();
                    Some(ScreenState::Night)
                } else if state_countdown.elapsed() > Duration::from_secs(5) {
                    state_countdown = Instant::now();
                    Some(ScreenState::Day(msg.next()))
                } else {
                    None
                }
            },
            &ScreenState::Night => {
                if touched {
                    night_cooldown = Instant::now();
                    Some(ScreenState::Day(Message::Date))
                } else if !check_night(start_night, end_night) {
                    Some(ScreenState::Day(Message::Date))
                } else {
                    None
                }
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
                                         &Color::new_4byte(0, 0, 0, 170));

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

                /*font.draw(&format!("FPS: {}", counter.tick()),
                          &Color::new_3byte(255, 255, 255),
                          20, &Position::new(20, 50), &mut drawer);*/
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

                if state_countdown.elapsed() > Duration::from_secs(5) || night_x == -1  {
                    state_countdown = Instant::now();

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
        thread::sleep(Duration::from_millis(1000));
        //drawer.vsync();
    }

    println!("Begin shutdown!");
}

fn main() {
    println!("Leaffront {}", VERSION);

    let context = Context::build().unwrap();

    gl_loop(context);
}
