extern crate leaffront_core;
extern crate leaffront_render_glutin;

extern crate glutin;

use leaffront_core::input::Input;
use leaffront_core::version::VersionInfo;

use leaffront_render_glutin::drawer::GlutinDrawer;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

use crate::glutin::prelude::GlSurface;

use std::num::NonZero;
use std::time::Instant;

pub struct GlutinInput {
    mouse_down: bool,
    mouse_x: usize,
    mouse_y: usize,
    running: bool,
}

/// The inner worker wraps winit events
struct InnerWorker<'a, T: FnMut(&GlutinInput, &mut GlutinDrawer) -> (bool, Instant) + 'static> {
    drawer: GlutinDrawer,
    function: T,
    next_time: Instant,
    input_engine: &'a mut GlutinInput,
}

impl<T: FnMut(&GlutinInput, &mut GlutinDrawer) -> (bool, Instant) + 'static> ApplicationHandler
    for InnerWorker<'_, T>
{
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        // We don't support resumed
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.input_engine.running = false;

                // Enable any final cleanup
                (self.function)(&self.input_engine, &mut self.drawer);

                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let (result, wait_till) = (self.function)(&self.input_engine, &mut self.drawer);

                self.next_time = wait_till;

                if !result {
                    event_loop.exit();
                } else {
                    event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_time.clone()));
                }
            }
            WindowEvent::MouseInput { state, .. } => {
                self.input_engine.mouse_down = state == ElementState::Pressed;
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y): (i32, i32) = position.into();
                self.input_engine.mouse_x = x as usize;
                self.input_engine.mouse_y = y as usize;
            }
            WindowEvent::Resized(physical_size) => self.drawer.gl_surface.resize(
                &self.drawer.context,
                NonZero::new(physical_size.width).expect("Not zero!"),
                NonZero::new(physical_size.height).expect("Not zero!"),
            ),
            _ => (),
        }
    }
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

    fn run<T: FnMut(&Self, &mut Self::Window) -> (bool, Instant) + 'static>(
        mut self,
        mut drawer: Self::Window,
        function: T,
    ) {
        let event_loop = drawer
            .events_loop
            .take()
            .expect("Should have an events loop to run!");

        let mut worker = InnerWorker {
            drawer,
            function,
            next_time: Instant::now(),
            input_engine: &mut self,
        };

        event_loop.set_control_flow(ControlFlow::Wait);

        event_loop.run_app(&mut worker).expect("Failed to run app");
    }

    /// Checks to see if the mouse/pointer is down
    fn is_mouse_down(&self) -> bool {
        self.mouse_down
    }

    fn get_mouse_pos(&self) -> (usize, usize) {
        (self.mouse_x, self.mouse_y)
    }

    fn do_continue(&self) -> bool {
        self.running
    }
}

impl VersionInfo for GlutinInput {
    fn version() -> String {
        format!("glutin ({})", env!("CARGO_PKG_VERSION"))
    }
}
