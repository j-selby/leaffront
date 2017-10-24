use super::render::Drawer;

/// Handles basic input
pub trait Input {
    type Window : Drawer;

    /// Updates input
    fn update(&mut self, drawer : &mut Self::Window);

    /// Checks to see if the mouse/pointer is down
    fn is_mouse_down(&self) -> bool;

    /// Checks to see if execution should be continued
    fn do_continue(&self) -> bool;
}
