extern crate leaffront_core;
extern crate leaffront_render_glutin;

use leaffront_core::input::Input;

use leaffront_render_glutin::drawer::GlutinDrawer;

pub struct GlutinInput {
    mouse_down : bool
}

impl GlutinInput {
    pub fn new() -> Self {
        GlutinInput {
            mouse_down : false
        }
    }
}

impl Input for GlutinInput {
    type Window = GlutinDrawer;

    /// Updates input
    fn update(&mut self, _ : &Self::Window) {
        // TODO!
    }

    /// Checks to see if the mouse/pointer is down
    fn is_mouse_down(&self) -> bool {
        self.mouse_down
    }
}
