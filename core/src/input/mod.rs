use super::render::Drawer;

use std::time::Instant;

/// Handles basic input
pub trait Input {
    type Window: Drawer;

    /// Infinitely runs the loop until told otherwise
    fn run<T: FnMut(&Self, &mut Self::Window) -> (bool, Instant) + 'static>(
        self,
        drawer: Self::Window,
        function: T,
    );

    /// Checks to see if the mouse/pointer is down
    fn is_mouse_down(&self) -> bool;

    /// Returns the current mouse position in a (x, y) tuple.
    fn get_mouse_pos(&self) -> (usize, usize);

    /// Checks to see if execution should be continued
    fn do_continue(&self) -> bool;
}
