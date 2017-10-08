extern crate leaffront_core;
extern crate leaffront_render_pi;
extern crate evdev;

use leaffront_core::input::Input;
use leaffront_render_pi::drawer::PiDrawer;


/// Implements a basic input mechanism for the Pi through evdev.
pub struct PiInput {
    devices : Vec<evdev::Device>,
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
            mouse_down : false
        }
    }
}

impl Input for PiInput {
    type Window = PiDrawer;

    /// Updates input
    fn update(&mut self, _ : &Self::Window) {
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
}