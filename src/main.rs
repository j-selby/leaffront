extern crate leaffront_core;
extern crate leaffront_weather;

#[cfg(feature = "raspberry_pi")]
extern crate leaffront_render_pi;
#[cfg(feature = "raspberry_pi")]
extern crate leaffront_input_pi;

#[cfg(feature = "glutin")]
extern crate leaffront_render_glutin;
#[cfg(feature = "glutin")]
extern crate leaffront_input_glutin;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;

extern crate clap;

extern crate image;

extern crate chrono;
extern crate rand;

extern crate ctrlc;

mod state;
mod config;

mod background;
mod watchdog;

use config::LeaffrontConfig;

use leaffront_core::render::color::Color;
use leaffront_core::render::font::FontCache;
use leaffront_core::render::Drawer;
use leaffront_core::pos::Position;
use leaffront_core::pos::Rect;
use leaffront_core::input::Input;

use leaffront_weather::manager::WeatherManager;

#[cfg(feature = "raspberry_pi")]
use leaffront_render_pi::drawer::PiDrawer as DrawerImpl;
#[cfg(feature = "raspberry_pi")]
use leaffront_input_pi::PiInput as InputImpl;

#[cfg(feature = "glutin")]
use leaffront_render_glutin::drawer::GlutinDrawer as DrawerImpl;
#[cfg(feature = "glutin")]
use leaffront_input_glutin::GlutinInput as InputImpl;

use background::manager::BackgroundManager;

use state::ScreenState;
use state::Message;
use watchdog::Watchdog;

use clap::{Arg, App};

use chrono::Local;
use chrono::Datelike;
use chrono::naive::NaiveTime;
use chrono::Duration as cDuration;

use rand::Rng;
use rand::thread_rng;

use std::thread;
use std::time::Duration;
use std::time::Instant;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub static VERSION : &'static str = "2.0.0";

fn check_night(start_night : u32, end_night : u32) -> bool {
    let time = Local::now();
    let start_time = NaiveTime::from_hms(start_night, 0, 0);
    let end_time = NaiveTime::from_hms(end_night, 0, 0);
    let cur_date = time.naive_local();
    let cur_time = cur_date.time();

    let start_date = if (cur_time < end_time) && !(start_time < end_time) {
        // Early morning
        let start_date = time.date().naive_local();
        let start_date = start_date - cDuration::days(1);
        start_date.and_time(start_time)
    } else {
        time.date().naive_local().and_time(start_time)
    };

    let end_date = if start_time > end_time && !(cur_time < end_time) {
        // End night is on the next day
        let end_date = time.date().naive_local();
        let end_date = end_date + cDuration::days(1);
        end_date.and_time(end_time)
    } else {
        time.date().naive_local().and_time(end_time)
    };

    cur_date > start_date && cur_date < end_date
}

fn main_loop(config : LeaffrontConfig) {
    let start_night = config.sleep.sleep_hour;
    let end_night = config.sleep.wakeup_hour;

    let watchdog = Watchdog::build();

    // Create our mechanism for rendering
    let mut drawer = DrawerImpl::new();

    // Startup input handling
    let mut input = InputImpl::new();

    // Check the startup time
    let mut state = if check_night(start_night, end_night) {
        ScreenState::Night
    } else {
        ScreenState::Day(Message::Date)
    };

    let brightness = match state {
        ScreenState::Day(_) => config.day.brightness,
        ScreenState::Night => config.night.brightness,
    };
    drawer.set_brightness(brightness).unwrap();

    let mut state_countdown = Instant::now();

    let font_data = include_bytes!("../res/opensans.ttf");

    let mut font = FontCache::from_bytes(font_data);

    let mut weather_manager =
        WeatherManager::new(config.weather.update_freq * 60 * 1000);

    let mut rng = thread_rng();
    let mut night_x = -1;
    let mut night_y = -1;
    let mut night_cooldown = Instant::now();

    // Update the background
    let bg_mgr = BackgroundManager::new(config.art_dir);
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

    println!("Initialised successfully");

    // TODO: Mechanism to immediately wake up loop
    while running.load(Ordering::SeqCst) {
        watchdog.ping();

        input.update(&drawer);

        let touched = input.is_mouse_down();

        let next_img = bg_mgr.get_next();
        match next_img {
            Some(img) => {
                drawer.set_background(img);
            }
            _ => {}
        }

        let next_state = match &state {
            &ScreenState::Day(ref msg) => {
                if bg_countdown.elapsed() > Duration::from_secs(config.day.background_secs) {
                    bg_countdown = Instant::now();
                    bg_mgr.next();
                }

                if night_cooldown.elapsed() > Duration::from_secs(config.night.night_tap_cooldown) &&
                    check_night(start_night, end_night) {
                    state_countdown = Instant::now();
                    Some(ScreenState::Night)
                } else if state_countdown.elapsed() > Duration::from_secs(config.day.subtitle_secs) {
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
                let brightness = match state {
                    ScreenState::Day(_) => config.day.brightness,
                    ScreenState::Night => config.night.brightness,
                };
                drawer.set_brightness(brightness).unwrap();
            }
            None => {}
        }

        drawer.start();
        let screen_width = drawer.get_width() as i32;
        let screen_height = drawer.get_height() as i32;

        match &state {
            &ScreenState::Day(ref subtitle) => {
                drawer.clear(true);
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

                if state_countdown.elapsed() > Duration::from_secs(config.night.move_secs)
                    || night_x == -1  {
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
        thread::sleep(Duration::from_millis(config.refresh_rate));
        //drawer.vsync();
    }

    println!("Begin shutdown!");
}

fn main() {
    let matches = App::new("Leaffront")
        .version(VERSION)
        .author("Selby (https://github.com/j-selby)")
        .about("A simple photoframe for the Raspberry Pi")
        .long_about("Leaffront uses DispmanX + OpenGL to provide a simple slideshow, \
                            along with basic clock, date and weather information. \
                            Most values can be configured, and is lightweight enough that other \
                            applications can be run alongside to enhance the experience.")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .help("Provide a custom configuration file")
            .default_value("config.toml")
            .value_name("FILE")
            .required(false)
            .takes_value(true))
        .get_matches();

    let config_file = matches.value_of("config").unwrap_or("config.toml");

    println!("Leaffront {}", VERSION);

    let config = config::load_config(config_file.into());

    main_loop(config);
}
