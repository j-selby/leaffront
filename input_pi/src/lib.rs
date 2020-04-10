extern crate leaffront_core;
extern crate leaffront_render_pi;

extern crate evdev;

use leaffront_core::input::Input;
use leaffront_core::version::VersionInfo;

use evdev::{ABSOLUTE, ABS_X, ABS_Y};
use leaffront_render_pi::drawer::PiDrawer;
use std::time::{Duration, Instant};
use std::{process, thread};

/// Implements a basic input mechanism for the Pi through evdev.
pub struct PiInput {
    devices: Vec<evdev::Device>,
    mouse_x: usize,
    mouse_y: usize,
    mouse_down: bool,
}

impl PiInput {
    fn detect_devices(&mut self) {
        self.devices.clear();

        let devices = evdev::enumerate();

        for device in devices {
            if device.events_supported().contains(ABSOLUTE) {
                println!("Found input device: {:?}", device.name());
                self.devices.push(device);
            }
        }
    }

    /// Updates input
    fn update(&mut self) {
        let mut input = Vec::new();

        // Rust's retain doesn't allow for mutable access, so do this manually
        let mut i = 0;
        while i < self.devices.len() {
            let device = &mut self.devices[i];

            // Grab the devices name, then read events from it
            match device.name().to_str().map(|x| x.to_string()) {
                Ok(device_name) => match device.events_no_sync() {
                    Ok(events) => {
                        for evt in events {
                            input.push(evt);
                        }
                        i += 1;
                        continue;
                    }
                    Err(e) => {
                        println!("Device {:?} failed to send events: {:?}", device_name, e);
                    }
                },
                Err(e) => println!("Failed to read device name: {:?}", e),
            }

            let _ = self.devices.remove(i);
        }

        let mut touched = false;

        // Many events come from evdev devices - handle them
        for input in input {
            if input._type == ABSOLUTE.number() {
                touched = true;
                if input.code == ABS_X.number() {
                    self.mouse_x = input.value as usize;
                } else if input.code == ABS_Y.number() {
                    self.mouse_y = input.value as usize;
                }
            }
        }

        self.mouse_down = touched;
    }

    pub fn new() -> Self {
        let mut input = PiInput {
            devices: Vec::new(),
            mouse_x: 0,
            mouse_y: 0,
            mouse_down: false,
        };

        input.detect_devices();

        input
    }
}

impl Input for PiInput {
    type Window = PiDrawer;

    fn run<T: FnMut(&Self, &mut Self::Window) -> (bool, Instant) + 'static>(
        mut self,
        mut drawer: Self::Window,
        mut function: T,
    ) -> ! {
        loop {
            self.update();

            let (do_continue, wait_for) = function(&mut self, &mut drawer);
            if !do_continue {
                break;
            }

            let duration = wait_for - Instant::now();
            // Don't wait on negative times
            if duration > Duration::default() {
                thread::sleep(duration);
            }
        }

        process::exit(0)
    }

    /// Checks to see if the mouse/pointer is down
    fn is_mouse_down(&self) -> bool {
        self.mouse_down
    }

    fn get_mouse_pos(&self) -> (usize, usize) {
        (self.mouse_x, self.mouse_y)
    }

    // No way of telling this
    fn do_continue(&self) -> bool {
        true
    }
}

impl VersionInfo for PiInput {
    fn version() -> String {
        format!("evdev ({})", env!("CARGO_PKG_VERSION"))
    }
}
