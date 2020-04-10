use leaffront_core::backend::Backend;
use leaffront_core::input::Input;
use leaffront_core::pos::Position;
use leaffront_core::pos::Rect;
use leaffront_core::render::color::Color;
use leaffront_core::render::font::FontCache;
use leaffront_core::render::Drawer;

use leaffront_weather::manager::WeatherManager;

use leaffront_ui::*;

use background::manager::BackgroundManager;

use state::DisplayNotification;
use state::Message;
use state::ScreenState;

use chrono::Datelike;
use chrono::Local;

use rand::thread_rng;
use rand::Rng;

use std::thread;
use std::time::Duration;
use std::time::Instant;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use clock::check_night;

use config::LeaffrontConfig;

use platform::*;

use ctrlc;

pub fn main_loop(config: LeaffrontConfig) {
    let start_night = config.sleep.sleep_hour;
    let end_night = config.sleep.wakeup_hour;

    // Connect to the backend
    let mut backend = BackendImpl::new().unwrap();

    let mut notifications = Vec::new();

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
    match drawer.set_brightness(brightness) {
        Err(v) => println!("Failed to set brightness: {:?}", v),
        _ => {}
    }

    let mut state_countdown = Instant::now();

    let font_data = include_bytes!("../res/Lato-Regular.ttf");

    let mut font = FontCache::from_bytes(font_data);

    let mut weather_manager = WeatherManager::new(
        config.weather.update_freq * 60 * 1000,
        config.weather.kind,
        config.weather.config
    );

    let mut rng = thread_rng();
    let mut night_x = -1;
    let mut night_y = -1;
    let mut night_cooldown = Instant::now();

    let mut top_touch_cooldown = Instant::now();
    let mut show_top_touch = false;

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

    while running.load(Ordering::SeqCst) {
        input.update(&mut drawer);

        if !input.do_continue() {
            break;
        }

        // Handle incoming notifications
        match backend.get_notification() {
            Some(notify) => {
                notifications.push(DisplayNotification::new(notify));
            }
            None => {}
        }

        // Tick notifications
        // TODO: Config time
        notifications.retain(|x| x.displayed.elapsed() < Duration::from_secs(5));

        // Handle the adjustment of state
        let touched = input.is_mouse_down();

        let next_img = bg_mgr.get_next();
        match next_img {
            Some(img) => {
                drawer.set_background(img);
            }
            _ => {}
        }

        /*if touched {
            top_touch_cooldown = Instant::now();
            show_top_touch = true;
        }

        if Instant::now() - Duration::from_secs(5) > top_touch_cooldown {
            show_top_touch = false;
        }*/

        println!("{:?}", input.get_mouse_pos());

        let next_state = match &state {
            &ScreenState::Day(ref msg) => {
                if touched {
                    night_cooldown = Instant::now();
                }

                if bg_countdown.elapsed() > Duration::from_secs(config.day.background_secs) {
                    bg_countdown = Instant::now();
                    bg_mgr.next();
                }

                if night_cooldown.elapsed() > Duration::from_secs(config.night.night_tap_cooldown)
                    && check_night(start_night, end_night)
                {
                    state_countdown = Instant::now();
                    Some(ScreenState::Night)
                } else if state_countdown.elapsed() > Duration::from_secs(config.day.subtitle_secs)
                {
                    state_countdown = Instant::now();
                    Some(ScreenState::Day(msg.next()))
                } else {
                    None
                }
            }
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
                match drawer.set_brightness(brightness) {
                    Err(v) => println!("Failed to set brightness: {:?}", v),
                    _ => {}
                }
            }
            None => {}
        }

        drawer.start();

        match &state {
            &ScreenState::Day(..) => {
                drawer.clear(true);
            }
            &ScreenState::Night => {
                drawer.clear(false);
            }
        }

        drawer.enable_blending();

        let screen_width = drawer.get_width();
        let screen_height = drawer.get_height();

        if let Some(mut root) = begin_root(&mut drawer, vec![&mut font], (screen_width, screen_height)) {

            match &state {
                &ScreenState::Day(ref subtitle) => {
                    root.style.window.background = Color::new_4byte(0, 0, 0, 170);
                    root.style.text.color = Color::new_3byte(255, 255, 255);
                    root.style.text.size = 50;

                    let window_height = 120f32 / (screen_height as f32);

                    if let Some(mut window) = root.begin_window(
                        WindowOptions {
                            position: (0f32, 1.0f32 - window_height),
                            size: (1.0, window_height),
                            decorations: false,
                            ..WindowOptions::default()
                        }
                    ) {
                        let datetime = Local::now();
                        let time = datetime.format("%-I:%M:%S %P").to_string();
                        let msg = format!("{}", time);

                        window.text(msg);

                        match subtitle {
                            &Message::Date => {
                                let suffix = match datetime.day() {
                                    1 | 21 | 31 => "st",
                                    2 | 22 => "nd",
                                    3 | 23 => "rd",
                                    _ => "th",
                                };

                                let msg = format!(
                                    "{}{} of {}",
                                    datetime.format("%A, %-d").to_string(),
                                    suffix,
                                    datetime.format("%B").to_string()
                                );

                                window.text(msg);
                            }
                            &Message::Weather => {
                                let weather = weather_manager.get();
                                let msg = match weather {
                                    Ok(weather) => {
                                        format!("{}°C - {}", weather.temperature.round(), weather.description)
                                    }
                                    Err(msg) => msg,
                                };


                                window.text(msg);
                            }
                        }
                    }
                }
                &ScreenState::Night => {
                    /*drawer.clear(false);

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

                    let bottom_msg = format!(
                        "{}{} of {}",
                        datetime.format("%A, %-d").to_string(),
                        suffix,
                        datetime.format("%B").to_string()
                    );
                    let bottom_length = font.get_width(&bottom_msg, 50);
                    let bottom_two = bottom_length / 2;

                    if state_countdown.elapsed() > Duration::from_secs(config.night.move_secs)
                        || night_x == -1
                    {
                        state_countdown = Instant::now();

                        // Set new random position
                        // Calculate maximum ranges
                        let max_width = if top_two > bottom_two {
                            top_two
                        } else {
                            bottom_two
                        };
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

                    font.draw(
                        &top_msg,
                        &Color::new_3byte(255, 255, 255),
                        50,
                        &Position::new(top_x, night_y),
                        &mut drawer,
                    );
                    font.draw(
                        &bottom_msg,
                        &Color::new_3byte(255, 255, 255),
                        50,
                        &Position::new(bottom_x, night_y + 50),
                        &mut drawer,
                    );*/
                }
            }
        }

        // Begin render
        /*drawer.start();
        let screen_width = drawer.get_width() as i32;
        let screen_height = drawer.get_height() as i32;

        match &state {
            &ScreenState::Day(ref subtitle) => {
                drawer.clear(true);
                drawer.enable_blending();

                drawer.draw_colored_rect(
                    &Rect::new(0, screen_height - 120, screen_width, 120),
                    &Color::new_4byte(0, 0, 0, 170),
                );

                let datetime = Local::now();
                let time = datetime.format("%-I:%M:%S %P").to_string();
                let msg = format!("{}", time);

                font.draw(
                    &msg,
                    &Color::new_3byte(255, 255, 255),
                    50,
                    &Position::new(20, screen_height - 75),
                    &mut drawer,
                );

                match subtitle {
                    &Message::Date => {
                        let suffix = match datetime.day() {
                            1 | 21 | 31 => "st",
                            2 | 22 => "nd",
                            3 | 23 => "rd",
                            _ => "th",
                        };

                        let msg = format!(
                            "{}{} of {}",
                            datetime.format("%A, %-d").to_string(),
                            suffix,
                            datetime.format("%B").to_string()
                        );

                        font.draw(
                            &msg,
                            &Color::new_3byte(255, 255, 255),
                            50,
                            &Position::new(20, screen_height - 25),
                            &mut drawer,
                        );
                    }
                    &Message::Weather => {
                        let weather = weather_manager.get();
                        let msg = match weather {
                            Ok(weather) => {
                                format!("{}°C - {}", weather.temperature.round(), weather.description)
                            }
                            Err(msg) => msg,
                        };

                        font.draw(
                            &msg,
                            &Color::new_3byte(255, 255, 255),
                            50,
                            &Position::new(20, screen_height - 25),
                            &mut drawer,
                        );
                    }
                }

                if show_top_touch {
                    drawer.draw_colored_rect(
                        &Rect::new(0, 0, screen_width, 60),
                        &Color::new_4byte(0, 0, 0, 170),
                    );

                    font.draw(
                        "Exit",
                        &Color::new_3byte(255, 255, 255),
                        30,
                        &Position::new(20, 15),
                        &mut drawer,
                    )
                }
            }
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

                let bottom_msg = format!(
                    "{}{} of {}",
                    datetime.format("%A, %-d").to_string(),
                    suffix,
                    datetime.format("%B").to_string()
                );
                let bottom_length = font.get_width(&bottom_msg, 50);
                let bottom_two = bottom_length / 2;

                if state_countdown.elapsed() > Duration::from_secs(config.night.move_secs)
                    || night_x == -1
                {
                    state_countdown = Instant::now();

                    // Set new random position
                    // Calculate maximum ranges
                    let max_width = if top_two > bottom_two {
                        top_two
                    } else {
                        bottom_two
                    };
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

                font.draw(
                    &top_msg,
                    &Color::new_3byte(255, 255, 255),
                    50,
                    &Position::new(top_x, night_y),
                    &mut drawer,
                );
                font.draw(
                    &bottom_msg,
                    &Color::new_3byte(255, 255, 255),
                    50,
                    &Position::new(bottom_x, night_y + 50),
                    &mut drawer,
                );
            }
        }*/

        // Draw notifications
        let mut y = 50;
        let x = drawer.get_width() as i32 - 300 - 50;
        for notification in &notifications {
            drawer.draw_colored_rect(&Rect::new(x, y, 300, 100), &Color::new_4byte(0, 0, 0, 170));
            font.draw(
                &notification.source.name,
                &Color::new_3byte(255, 255, 255),
                30,
                &Position::new(x + 10, y + 20),
                &mut drawer,
            );
            font.draw(
                &notification.source.contents,
                &Color::new_3byte(255, 255, 255),
                20,
                &Position::new(x + 10, y + 40),
                &mut drawer,
            );
            y += 120;
        }

        drawer.end();

        thread::sleep(Duration::from_millis(config.refresh_rate));
    }
}
