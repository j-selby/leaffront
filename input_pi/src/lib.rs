extern crate leaffront_core;
extern crate leaffront_render_pi;

extern crate evdev;

use leaffront_core::input::Input;
use leaffront_core::version::VersionInfo;

use evdev::{DeviceState, RawEvents, ABSOLUTE, ABS_X, ABS_Y};
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
        self.devices
            .retain(|mut device| match device.events_no_sync() {
                Ok(events) => {
                    for evt in events {
                        input.push(evt);
                    }
                    true
                }
                Err(e) => {
                    println!("Device {:?} failed to send events: {:?}", device.name(), e);
                    false
                }
            });

        let mut touched = false;

        for input in input {
            if input._type == ABSOLUTE.number() {
                touched = true;
                if ABS_X.intersects(input.code) {
                    println!("Got X abs event!");
                    self.mouse_x = input.value as usize;
                } else if ABS_Y.intersects(input.code) {
                    println!("Got Y abs event!");
                    self.mouse_y = input.value as usize;
                } else {
                    println!("Got known event {}!", input.code);
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
        function: T,
    ) -> ! {
        loop {
            self.update();

            let (do_continue, wait_for) = function(&mut self, &mut drawer);
            if !do_continue {
                break;
            }

            let duration = wait_for - Instant::now();
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
