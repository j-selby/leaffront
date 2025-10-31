extern crate leaffront_core;
extern crate leaffront_render_glutin;

extern crate glutin;

use leaffront_core::input::Input;
use leaffront_core::version::VersionInfo;

use leaffront_render_glutin::drawer::GlutinDrawer;

use glutin::event::Event;
use glutin::event::WindowEvent;
use glutin::event_loop::ControlFlow;

use std::time::Instant;

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

    fn run<T: FnMut(&Self, &mut Self::Window) -> (bool, Instant) + 'static>(
        mut self,
        mut drawer: Self::Window,
        mut function: T,
    ) -> ! {
        let events = drawer
            .events_loop
            .take()
            .expect("Should have an events loop to run!");

        let mut next_time = Instant::now();

        events.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::WaitUntil(next_time.clone());

            match event {
                Event::LoopDestroyed => return,
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        self.running = false;
                    }
                    WindowEvent::MouseInput { state, .. } => {
                        self.mouse_down = state == glutin::event::ElementState::Pressed;
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let (x, y): (i32, i32) = position.into();
                        self.mouse_x = x as usize;
                        self.mouse_y = y as usize;
                    }
                    WindowEvent::Resized(physical_size) => drawer.gl_window.resize(physical_size),
                    _ => (),
                },
                Event::RedrawRequested(_) => {
                    let (result, wait_till) = function(&self, &mut drawer);
                    next_time = wait_till;
                    if !result {
                        *control_flow = ControlFlow::Exit;
                    } else {
                        *control_flow = ControlFlow::WaitUntil(next_time.clone());
                    }
                }
                _ => (),
            }

            if Instant::now() >= next_time {
                drawer.gl_window.window().request_redraw();
            }
        })
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
