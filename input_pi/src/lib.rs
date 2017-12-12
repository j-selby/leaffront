extern crate leaffront_core;
extern crate leaffront_render_pi;

extern crate evdev;

use leaffront_core::input::Input;
use leaffront_core::version::VersionInfo;

use leaffront_render_pi::drawer::PiDrawer;

/// Implements a basic input mechanism for the Pi through evdev.
pub struct PiInput {
    devices : Vec<evdev::Device>,
    mouse_x : usize,
    mouse_y : usize,
    mouse_down : bool
}

impl PiInput {
    pub fn new() -> Self {
        let devices = evdev::enumerate();

        for device in &devices {
            println!("Found input device: {:?}", device.name());
        }

        PiInput {
            devices,
            mouse_x : 0,
            mouse_y : 0,
            mouse_down : false
        }
    }
}

impl Input for PiInput {
    type Window = PiDrawer;

    /// Updates input
    fn update(&mut self, _ : &mut Self::Window) {
        let mut input = Vec::new();
        for device  in &mut self.devices {
            for evt in  device.events_no_sync().unwrap() {
                input.push(evt);
            }
        }

        let mut touched = false;

        for input in input {
            if input._type == 3 {
                touched = true;
                break;
            }
        }

        self.mouse_down = touched;
    }

    /// Checks to see if the mouse/pointer is down
    fn is_mouse_down(&self) -> bool {
        self.mouse_down
    }

    // No way of telling this
    fn do_continue(&self) -> bool {
        true
    }

    fn get_mouse_pos(&self) -> (usize, usize) {
        unimplemented!()
    }
}

impl VersionInfo for PiInput {
    fn version() -> String {
        format!("evdev ({})", env!("CARGO_PKG_VERSION"))
    }
}
