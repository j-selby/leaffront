use leaffront_core::backend::Backend;
use leaffront_core::input::Input;
use leaffront_core::pos::Rect;
use leaffront_core::render::color::Color;
use leaffront_core::render::texture::Texture;
use leaffront_core::render::Drawer;

use leaffront_weather::manager::WeatherManager;

use crate::background::manager::BackgroundManager;

use crate::state::DisplayNotification;
use crate::state::Message;
use crate::state::ScreenState;

use crate::clock::check_night;

use crate::config::LeaffrontConfig;

use crate::platform::*;

use chrono::Datelike;
use chrono::Local;

use rand::thread_rng;
use rand::Rng;

use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use ctrlc;

use egui::epaint::Primitive;
use egui::style::Margin;
use egui::ClippedPrimitive;
use egui::TextureId;
use egui::TexturesDelta;
use egui::{Align2, Color32, Event, Frame, PointerButton, Pos2};

/// A texture bundle contains both a raw, CPU-managed texture, as well
/// as a GPU texture. This allows for updates to the CPU-managed texture
/// easily.
struct TextureBundle {
    sample: Texture,
    native: <DrawerImpl as Drawer>::NativeTexture,
}

pub fn main_loop(config: LeaffrontConfig) {
    let start_night = config.sleep.sleep_hour;
    let end_night = config.sleep.wakeup_hour;

    // Connect to the backend
    let mut backend = BackendImpl::new().unwrap();

    let mut notifications = Vec::new();

    // Create our mechanism for rendering
    let mut drawer = DrawerImpl::new();

    // Startup input handling
    let input = InputImpl::new();

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
        Err(v) => warn!("Failed to set brightness: {:?}", v),
        _ => {}
    }

    let mut state_countdown = Instant::now();

    //let font_data = include_bytes!("../res/Lato-Regular.ttf");

    let mut weather_manager = WeatherManager::new(
        config.weather.update_freq * 60 * 1000,
        config.weather.kind,
        config.weather.config.clone(),
    );

    let mut rng = thread_rng();
    let mut night_x = -1f32;
    let mut night_y = -1f32;
    let mut night_cooldown = Instant::now();

    // Update the background
    let bg_mgr = BackgroundManager::new(config.art_dir.clone());
    let mut bg_countdown = Instant::now();
    bg_mgr.next();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    let has_called = Arc::new(AtomicBool::new(false));

    ctrlc::set_handler(move || {
        info!("Ctrl-C received");
        if has_called.load(Ordering::SeqCst) {
            warn!("Forcing shutdown:");
            ::std::process::exit(1);
        } else {
            has_called.store(true, Ordering::SeqCst);
            r.store(false, Ordering::SeqCst);
        }
    })
    .expect("Error setting Ctrl-C handler");

    let egui_ctx = egui::Context::default();

    let mut style = egui_ctx.style().as_ref().to_owned();
    style.spacing.window_margin = Margin::symmetric(15.0, 15.0);
    style.visuals.dark_mode = true;
    style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgba_unmultiplied(20, 20, 20, 220);
    style.visuals.widgets.noninteractive.bg_stroke.color =
        Color32::from_rgba_unmultiplied(20, 20, 20, 220);
    style.visuals.widgets.noninteractive.fg_stroke.color = Color32::WHITE;
    style
        .text_styles
        .get_mut(&egui::TextStyle::Heading)
        .expect("Heading text style should exist")
        .size += 20.0;
    egui_ctx.set_style(style);

    let start_time = Instant::now();

    let mut egui_textures: HashMap<TextureId, TextureBundle> = HashMap::new();

    drawer.set_fullscreen(config.fullscreen);

    let mut textures_delta = TexturesDelta::default();

    let mut last_second = Local::now();
    let mut last_set_brightness = None;

    let mut pointer_event: Option<Event> = None;

    info!("Initialised successfully");

    input.run(drawer, move |input, drawer| {
        if !running.load(Ordering::SeqCst) || !input.do_continue() {
            return (false, Instant::now());
        }

        // Translate the leaffront input into egui input
        let mut raw_input = egui::RawInput::default();
        raw_input.screen_rect = Some(egui::Rect {
            min: Pos2 { x: 0.0, y: 0.0 },
            max: Pos2 {
                x: drawer.get_width() as _,
                y: drawer.get_height() as _,
            },
        });
        raw_input.pixels_per_point = Some(1.0);
        raw_input.time = Some((Instant::now() - start_time).as_secs_f64());

        let (mouse_x, mouse_y) = input.get_mouse_pos();

        // Only send a mouse event if something has actually changed
        let new_pointer_event = Event::PointerButton {
            pos: Pos2::new(mouse_x as _, mouse_y as _),
            button: PointerButton::Primary,
            pressed: input.is_mouse_down(),
            modifiers: Default::default(),
        };

        if pointer_event.as_ref() != Some(&new_pointer_event) {
            raw_input.events.push(new_pointer_event.clone());
        }

        pointer_event = Some(new_pointer_event);

        // Redraw every second
        let mut dirty_state = false;
        if Local::now().timestamp() > last_second.timestamp() {
            dirty_state = true;
            last_second = Local::now();
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

        if let Some(next_img) = bg_mgr.get_next() {
            drawer.set_background(next_img);
            dirty_state = true;
        }

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
                dirty_state = true;

                // Configure brightness (if required)
                let brightness = match state {
                    ScreenState::Day(_) => config.day.brightness,
                    ScreenState::Night => config.night.brightness,
                };
                if last_set_brightness != Some(brightness) {
                    match drawer.set_brightness(brightness) {
                        Err(v) => warn!("Failed to set brightness: {:?}", v),
                        _ => {}
                    }

                    last_set_brightness = Some(brightness);
                }
            }
            None => {}
        }

        // Make sure egui recognises external updates
        if dirty_state {
            egui_ctx.request_repaint();
        }

        egui_ctx.begin_frame(raw_input);

        let screen_width = drawer.get_width();
        let screen_height = drawer.get_height();

        match &state {
            &ScreenState::Day(ref subtitle) => {
                let datetime = Local::now();

                egui::Window::new("Day Display")
                    .enabled(true)
                    .resizable(false)
                    .anchor(Align2::LEFT_BOTTOM, (10.0, -10.0))
                    .auto_sized()
                    .min_width(100.0)
                    .min_height(70.0)
                    .collapsible(false)
                    .title_bar(false)
                    .show(&egui_ctx, |ui| {
                        ui.heading(datetime.format("%-I:%M:%S %P").to_string());

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

                                ui.heading(msg);
                            }
                            &Message::Weather => {
                                let weather = weather_manager.get();
                                let msg = match weather {
                                    Ok(weather) => format!(
                                        "{}°C - {}",
                                        weather.temperature.round(),
                                        weather.description
                                    ),
                                    Err(msg) => msg,
                                };

                                ui.heading(msg);
                            }
                        }
                    });
            }
            &ScreenState::Night => {
                egui::Window::new("Night Display")
                    .enabled(true)
                    .resizable(false)
                    .anchor(Align2::CENTER_CENTER, (night_x, night_y))
                    .auto_sized()
                    .min_width(100.0)
                    .min_height(70.0)
                    .collapsible(false)
                    .title_bar(false)
                    .frame(Frame::none())
                    .show(&egui_ctx, |ui| {
                        // Render out both the top and bottom strings, and center them.
                        let datetime = Local::now();
                        let top_msg = datetime.format("%-I:%M:%S %P").to_string();

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

                        ui.vertical_centered(|ui| {
                            ui.heading(top_msg);
                            ui.heading(bottom_msg);
                        });
                    });

                if state_countdown.elapsed() > Duration::from_secs(config.night.move_secs) {
                    state_countdown = Instant::now();

                    // Set new random position
                    let max_x = screen_width as f32 - 200.0;
                    let min_x = 200.0;

                    // For gap:
                    let min_y = 200.0;
                    let max_y = screen_height as f32 - 200.0;

                    night_x = rng.gen_range(min_x..max_x) - screen_width as f32 / 2.0;
                    night_y = rng.gen_range(min_y..max_y) - screen_height as f32 / 2.0;
                }
            }
        }

        // Draw notifications
        for (i, notification) in notifications.iter().enumerate() {
            egui::Window::new(format!("Night Display {}", i))
                .enabled(true)
                .resizable(false)
                .anchor(Align2::RIGHT_TOP, (-10.0, 50.0 + (i as f32 * 120.0)))
                .auto_sized()
                .collapsible(false)
                .title_bar(false)
                .show(&egui_ctx, |ui| {
                    ui.heading(notification.source.name.to_owned());
                    ui.heading(notification.source.contents.to_owned());
                });
        }

        let output = egui_ctx.end_frame();

        textures_delta.append(output.textures_delta);

        if output.needs_repaint {
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

            // Upload all textures as required
            for (new_tex_id, image_delta) in &textures_delta.set {
                // Either create a new texture or fetch the old CPU (non-GPU) texture.
                let mut base_texture = match egui_textures.remove(new_tex_id) {
                    Some(texture_bundle) => texture_bundle.sample,
                    None => Texture::new(image_delta.image.width(), image_delta.image.height()),
                };

                let [base_x, base_y] = image_delta.pos.unwrap_or([0, 0]);

                match &image_delta.image {
                    egui::ImageData::Color(color_image) => {
                        for (i, pixel) in color_image.pixels.iter().enumerate() {
                            let y = base_y + (i / color_image.width());
                            let x = base_x + (i % color_image.width());
                            base_texture.draw_pixel(
                                &Color::new_4byte(pixel.r(), pixel.g(), pixel.b(), pixel.a()),
                                x,
                                y,
                            );
                        }
                    }
                    egui::ImageData::Font(font_image) => {
                        for (i, pixel) in font_image.srgba_pixels(1.0).enumerate() {
                            let y = base_y + (i / font_image.width());
                            let x = base_x + (i % font_image.width());
                            base_texture.draw_pixel(
                                &Color::new_4byte(pixel.r(), pixel.g(), pixel.b(), pixel.a()),
                                x,
                                y,
                            );
                        }
                    }
                }

                let native_texture =
                    <DrawerImpl as Drawer>::NativeTexture::from_texture(&base_texture);

                egui_textures.insert(
                    *new_tex_id,
                    TextureBundle {
                        sample: base_texture,
                        native: native_texture,
                    },
                );
            }

            for free_texture in &textures_delta.free {
                egui_textures.remove(free_texture);
            }

            textures_delta.clear();

            // Generate primitives we need to render

            let shapes = egui_ctx.tessellate(output.shapes);

            for ClippedPrimitive {
                clip_rect,
                primitive,
            } in shapes
            {
                // Translate the vertexes into points we can use
                let mut positions = Vec::with_capacity(16);
                let mut colors = Vec::with_capacity(24);
                let mut uv = Vec::with_capacity(16);

                let mesh = match primitive {
                    Primitive::Mesh(mesh) => mesh,
                    _ => continue,
                };

                for index in mesh.indices {
                    let vertex = &mesh.vertices[index as usize];
                    positions.push(vertex.pos.x / drawer.get_width() as f32 * 2.0 - 1.0);
                    positions.push((vertex.pos.y / drawer.get_height() as f32 * 2.0 - 1.0) * -1.0);

                    colors.push((vertex.color.r() as f32) / 255.0);
                    colors.push((vertex.color.g() as f32) / 255.0);
                    colors.push((vertex.color.b() as f32) / 255.0);
                    colors.push((vertex.color.a() as f32) / 255.0);

                    uv.push(vertex.uv.x);
                    uv.push(vertex.uv.y);
                }

                drawer.start_clip(&Rect::new(
                    clip_rect.min.x as _,
                    clip_rect.min.y as _,
                    (clip_rect.max.x - clip_rect.min.x) as _,
                    (clip_rect.max.y - clip_rect.min.y) as _,
                ));

                drawer.draw_textured_vertices_colored_uv(
                    &egui_textures[&mesh.texture_id].native,
                    positions.as_slice(),
                    colors.as_slice(),
                    uv.as_slice(),
                );

                drawer.end_clip();
            }

            drawer.end();
        }

        (
            true,
            Instant::now() + Duration::from_millis(config.refresh_rate),
        )
    });
}
