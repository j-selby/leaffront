extern crate leaffront_core;
extern crate leaffront_render_pi;

extern crate evdev;

#[macro_use]
extern crate log;

use leaffront_core::input::Input;
use leaffront_core::version::VersionInfo;

use evdev::AbsoluteAxisType;
use evdev::EventType;
use evdev::InputEventKind;

use leaffront_render_pi::drawer::PiDrawer;
use std::sync::mpsc::{channel, Receiver, RecvError, RecvTimeoutError, Sender, TryRecvError};
use std::time::{Duration, Instant};
use std::{process, thread};

/// Implements a basic input mechanism for the Pi through evdev.
struct PiInputThreaded {
    devices: Vec<evdev::Device>,
    mouse_x: usize,
    mouse_y: usize,
    mouse_down: bool,
    outgoing_channel: Sender<InputUpdate>,
}

struct InputUpdate {
    mouse_x: usize,
    mouse_y: usize,
    mouse_down: bool,
}

impl PiInputThreaded {
    fn detect_devices(&mut self) {
        self.devices.clear();

        let devices = evdev::enumerate();

        for device in devices {
            if device.supported_events().contains(EventType::ABSOLUTE) {
                info!("Found input device: {:?}", device.name());
                self.devices.push(device);
            }
        }
    }

    /// Updates input
    fn update(&mut self) -> bool {
        let mut input = Vec::new();

        // Rust's retain doesn't allow for mutable access, so do this manually
        let mut i = 0;
        while i < self.devices.len() {
            let device = &mut self.devices[i];

            // Grab the devices name, then read events from it
            match device.name().map(|x| x.to_owned()) {
                Some(device_name) => match device.fetch_events() {
                    Ok(events) => {
                        for evt in events {
                            input.push(evt);
                        }
                        i += 1;
                        continue;
                    }
                    Err(e) => {
                        warn!("Device {:?} failed to send events: {:?}", device_name, e);
                    }
                },
                None => warn!(
                    "Failed to read device name for {:?}",
                    device.physical_path()
                ),
            }

            let _ = self.devices.remove(i);
        }

        let mut touched = false;

        // Many events come from evdev devices - handle them
        for input in input {
            if input.event_type() == EventType::ABSOLUTE {
                touched = true;
                if input.kind() == InputEventKind::AbsAxis(AbsoluteAxisType::ABS_X) {
                    self.mouse_x = input.value() as usize;
                } else if input.kind() == InputEventKind::AbsAxis(AbsoluteAxisType::ABS_Y) {
                    self.mouse_y = input.value() as usize;
                }
            }
        }

        self.mouse_down = touched;

        if let Err(e) = self.outgoing_channel.send(InputUpdate {
            mouse_x: self.mouse_x,
            mouse_y: self.mouse_y,
            mouse_down: self.mouse_down,
        }) {
            warn!("Shutting down input thread: {:?}", e);
            false
        } else {
            true
        }
    }

    fn new(sender: Sender<InputUpdate>) -> Self {
        let mut input = PiInputThreaded {
            devices: Vec::new(),
            mouse_x: 0,
            mouse_y: 0,
            mouse_down: false,
            outgoing_channel: sender,
        };

        input.detect_devices();

        input
    }
}

pub struct PiInput {
    receiver: Receiver<InputUpdate>,
    mouse_x: usize,
    mouse_y: usize,
    mouse_down: bool,
}

impl PiInput {
    pub fn new() -> Self {
        let (sender, receiver) = channel();

        thread::spawn(|| {
            let mut input_device = PiInputThreaded::new(sender);

            while input_device.update() {}
        });

        PiInput {
            receiver,
            mouse_x: 0,
            mouse_y: 0,
            mouse_down: false,
        }
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
            let (do_continue, wait_for) = function(&mut self, &mut drawer);
            if !do_continue {
                break;
            }

            self.mouse_down = false;

            loop {
                let now = Instant::now();
                let duration = if now >= wait_for {
                    Duration::from_millis(1)
                } else {
                    wait_for - now
                };

                // Don't wait on small times
                let input = if duration <= Duration::from_millis(10) {
                    match self.receiver.try_recv() {
                        Ok(input) => input,
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => {
                            panic!("Failed to read input");
                        }
                    }
                } else {
                    match self.receiver.recv_timeout(duration) {
                        Ok(input) => input,
                        Err(RecvTimeoutError::Timeout) => break,
                        Err(RecvTimeoutError::Disconnected) => {
                            panic!("Failed to read input");
                        }
                    }
                };

                self.mouse_down = input.mouse_down;
                self.mouse_x = input.mouse_x;
                self.mouse_y = input.mouse_y;
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
