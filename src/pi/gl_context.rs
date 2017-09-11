/// Manages the configuration of a Raspberry Pi to get a GLES context.

// Code from https://github.com/seankerr/rust-rpi-examples

use egl;
use egl::{EGLConfig, EGLContext, EGLDisplay, EGLNativeDisplayType, EGLSurface};

use videocore::bcm_host;
use videocore::dispmanx;
use videocore::dispmanx::{FlagsAlpha, Transform, VCAlpha, Window, DisplayHandle, UpdateHandle,
                            ElementHandle};
use videocore::image::Rect;

use videocore::bcm_host::GraphicsDisplaySize;

use std::ptr;

pub struct Context {
    pub config:  EGLConfig,
    pub context: EGLContext,
    pub display: EGLDisplay,
    pub surface: EGLSurface,

    window : Box<Window>,
    dispman_display : DisplayHandle,
    update : UpdateHandle,
    element : ElementHandle
}

impl Context {
    /// Returns the screen resolution of the device.
    pub fn get_resolution() -> GraphicsDisplaySize {
        bcm_host::graphics_get_display_size(0).unwrap()
    }

    pub fn swap_buffers(&self) -> bool {
        egl::swap_buffers(self.display, self.surface)
    }

    pub fn build() -> Result<Self, String> {
        // first thing to do is initialize the broadcom host (when doing any graphics on RPi)
        bcm_host::init();

        // open the display
        let display = dispmanx::display_open(0);

        // get update handle
        let update = dispmanx::update_start(0);

        // get screen resolution (same display number as display_open()
        let dimensions : Result<GraphicsDisplaySize, String> =
            match bcm_host::graphics_get_display_size(0) {
                Some(x) => Ok(x),
                None => Err("bcm_host::init() did not succeed".into())
            };
        let dimensions = dimensions?;

        println!("Display size: {}x{}", dimensions.width, dimensions.height);

        // setup the destination rectangle where opengl will be drawing
        let mut dest_rect = Rect {
            x: 0,
            y: 0,
            width: dimensions.width as i32,
            height: dimensions.height as i32
        };

        // setup the source rectangle where opengl will be drawing
        let mut src_rect = Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0
        };

        // draw opengl context on a clean background (cleared by the clear color)
        let mut alpha = VCAlpha {
            flags: FlagsAlpha::FIXED_ALL_PIXELS,
            opacity: 255,
            mask: 0
        };

        // create our dispmanx element upon which we'll draw opengl using EGL
        let element = dispmanx::element_add(update, display,
                                  3, // layer upon which to draw
                                  &mut dest_rect,
                                  0,
                                  &mut src_rect,
                                  dispmanx::DISPMANX_PROTECTION_NONE,
                                  &mut alpha,
                                  ptr::null_mut(),
                                  Transform::NO_ROTATE);

        // submit changes
        dispmanx::update_submit_sync(update);

        // create window to hold element, width, height
        let mut window = Box::new( Window {
            element,
            width: dimensions.width as i32,
            height: dimensions.height as i32
        });

        // Create a EGL context
        let context_attr = [egl::EGL_CONTEXT_CLIENT_VERSION, 2,
            egl::EGL_NONE];

        let config_attr = [egl::EGL_RED_SIZE, 8,
            egl::EGL_GREEN_SIZE, 8,
            egl::EGL_BLUE_SIZE, 8,
            egl::EGL_ALPHA_SIZE, 8,
            egl::EGL_SURFACE_TYPE, egl::EGL_WINDOW_BIT,
            egl::EGL_NONE];

        // get display
        let egl_display : Result<EGLDisplay, String> =
            match egl::get_display(egl::EGL_DEFAULT_DISPLAY) {
            Some(x) => Ok(x),
            None => Err("Failed to get EGL display".into())
        };
        let egl_display : EGLDisplay = egl_display?;

        // init display
        if !egl::initialize(egl_display, &mut 0i32, &mut 0i32) {
            return Err("Failed to initialize EGL".into());
        }

        // choose first available configuration
        let egl_config : Result<EGLConfig, String> =
            match egl::choose_config(egl_display, &config_attr, 1) {
            Some(x) => Ok(x),
            None => Err("Failed to get EGL configuration".into())
        };
        let egl_config : EGLConfig = egl_config?;

        // bind opengl es api
        if !egl::bind_api(egl::EGL_OPENGL_ES_API) {
            return Err("Failed to bind EGL OpenGL ES API".into());
        }

        // create egl context
        let egl_context : Result<EGLContext, String> =
            match egl::create_context(egl_display, egl_config, egl::EGL_NO_CONTEXT,
                                                    &context_attr) {
            Some(x) => Ok(x),
            None => Err("Failed to create EGL context".into())
        };
        let egl_context : EGLContext = egl_context?;

        // create surface
        let egl_surface : Result<EGLSurface, String> =
            match egl::create_window_surface(egl_display, egl_config,
                                           window.as_mut() as *mut _ as EGLNativeDisplayType,
                                           &[]) {
            Some(x) => Ok(x),
            None => Err("Failed to create EGL surface".into())
        };
        let egl_surface : EGLSurface = egl_surface?;

        // set current context
        if !egl::make_current(egl_display, egl_surface, egl_surface, egl_context) {
            return Err("Failed to make EGL current context".into());
        }

        // remove the vsync/swap interval
        egl::swap_interval(egl_display, 0);

        Ok(Self {
            config: egl_config,
            context: egl_context,
            display: egl_display,
            surface: egl_surface,

            window,
            dispman_display : display,
            update,
            element
        })
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        println!("Context shutdown!");
        egl::destroy_surface(self.display, self.surface);
        egl::destroy_context(self.display, self.context);
        egl::terminate(self.display);

        dispmanx::element_remove(self.update, self.element);

        dispmanx::update_submit_sync(self.update);
        // "Update" cannot be deleted?

        if !dispmanx::display_close(self.dispman_display) {
            println!("Display shutdown successful.");
        } else {
            println!("Display shutdown failed.");
        }

        bcm_host::deinit();
    }
}
