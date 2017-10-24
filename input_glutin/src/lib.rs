extern crate leaffront_core;
extern crate leaffront_render_glutin;

extern crate glutin;

use glutin::GlContext;

use leaffront_core::input::Input;

use leaffront_render_glutin::drawer::GlutinDrawer;

pub struct GlutinInput {
    mouse_down : bool,
    running : bool
}

impl GlutinInput {
    pub fn new() -> Self {
        GlutinInput {
            mouse_down : false,
            running : true
        }
    }
}

impl Input for GlutinInput {
    type Window = GlutinDrawer;

    /// Updates input
    fn update(&mut self, window : &mut Self::Window) {
        let events = &mut window.events_loop;
        let window = &window.gl_window;
        events.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent{ event, .. } => match event {
                    glutin::WindowEvent::Closed => self.running = false,
                    glutin::WindowEvent::Resized(w, h) => window.resize(w, h),
                    glutin::WindowEvent::MouseInput {device_id, state, button} => {
                        self.mouse_down = state == glutin::ElementState::Pressed;
                    }
                    _ => ()
                },

                _ => ()
            }
        });
    }

    /// Checks to see if the mouse/pointer is down
    fn is_mouse_down(&self) -> bool {
        self.mouse_down
    }

    fn do_continue(&self) -> bool {
        self.running
    }
}
