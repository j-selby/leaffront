extern crate leaffront_core;
extern crate leaffront_render_glutin;

extern crate glutin;

use glutin::GlContext;

use leaffront_core::input::Input;
use leaffront_core::version::VersionInfo;

use leaffront_render_glutin::drawer::GlutinDrawer;

pub struct GlutinInput {
    mouse_down: bool,
    mouse_x: usize,
    mouse_y: usize,
    running: bool,
}

impl GlutinInput {
    pub fn new() -> Self {
        GlutinInput {
            mouse_down: false,
            mouse_x: 0,
            mouse_y: 0,
            running: true,
        }
    }
}

impl Input for GlutinInput {
    type Window = GlutinDrawer;

    /// Updates input
    fn update(&mut self, window: &mut Self::Window) {
        let events = &mut window.events_loop;
        let window = &window.gl_window;

        events.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => self.running = false,
                glutin::WindowEvent::Resized(size) => window.resize(size.to_physical(1.0)),
                glutin::WindowEvent::MouseInput { state, .. } => {
                    self.mouse_down = state == glutin::ElementState::Pressed;
                }
                glutin::WindowEvent::CursorMoved { position, .. } => {
                    let (x, y): (i32, i32) = position.into();
                    self.mouse_x = x as usize;
                    self.mouse_y = y as usize;
                }
                _ => (),
            },

            _ => (),
        });
    }

    /// Checks to see if the mouse/pointer is down
    fn is_mouse_down(&self) -> bool {
        self.mouse_down
    }

    fn do_continue(&self) -> bool {
        self.running
    }

    fn get_mouse_pos(&self) -> (usize, usize) {
        (self.mouse_x, self.mouse_y)
    }
}

impl VersionInfo for GlutinInput {
    fn version() -> String {
        format!("glutin ({})", env!("CARGO_PKG_VERSION"))
    }
}
