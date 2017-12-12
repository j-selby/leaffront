use leaffront_core::render::Drawer;
use leaffront_core::render::texture::Texture;

use image::DynamicImage;

use texture::GlTexture;

use shader::GLSLShader;

use vbo::GLVBO;

use leaffront_core::pos::Rect;
use leaffront_core::version::VersionInfo;
use leaffront_core::render::color::Color;

use glutin;
use glutin::GlContext;

use gl;

use std::ptr;
use std::mem;

use std::os::raw::c_void;
use std::os::raw::c_char;

#[derive(Ord, PartialOrd, Eq, PartialEq)]
enum DrawState {
    None,
    Colored,
    Textured
}

pub struct GlutinDrawer {
    pub events_loop : glutin::EventsLoop,
    pub gl_window : glutin::GlWindow,

    colored : GLSLShader,
    textured : GLSLShader,

    // Used by both shaders
    vertex : GLVBO,
    attr_colored_vertex : gl::types::GLint,
    attr_textured_vertex : gl::types::GLint,

    // Used by colored shader
    color : GLVBO,
    attr_colored_color : gl::types::GLint,
    attr_textured_color : gl::types::GLint,

    // Used by textured shader
    uv : GLVBO,
    attr_textured_uv : gl::types::GLint,

    // Background image
    background : Option<GlTexture>,

    state : DrawState,

    // Debugging
    transition_count : usize,
    calls : usize
}

impl GlutinDrawer {
    /// Changes shaders, and ensures that G is ready to use it.
    fn configure_state(&mut self, target : DrawState) {
        self.calls += 1;

        if self.state != target {
            self.transition_count += 1;

            // Unbind previous state, if needed
            match self.state {
                DrawState::None => {},
                DrawState::Colored => {
                    unsafe {
                        gl::DisableVertexAttribArray(self.attr_colored_vertex as gl::types::GLuint);
                        gl::DisableVertexAttribArray(self.attr_colored_color as gl::types::GLuint);
                    }
                },
                DrawState::Textured => {
                    unsafe {
                        gl::DisableVertexAttribArray(self.attr_textured_uv as gl::types::GLuint);
                        gl::DisableVertexAttribArray(self.attr_textured_vertex as gl::types::GLuint);
                        gl::DisableVertexAttribArray(self.attr_textured_color as gl::types::GLuint);
                    }
                }
            }

            // Configure new state
            match target {
                DrawState::None => {
                    unsafe {
                        gl::UseProgram(0);
                        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
                    }
                }
                DrawState::Colored => {
                    self.colored.use_program();

                    unsafe {
                        gl::EnableVertexAttribArray(self.attr_colored_vertex as gl::types::GLuint);
                        gl::EnableVertexAttribArray(self.attr_colored_color as gl::types::GLuint);

                        self.vertex.bind();
                        gl::VertexAttribPointer(self.attr_colored_vertex as gl::types::GLuint, 2,
                                                gl::FLOAT, false as gl::types::GLboolean,
                                                0, ptr::null());

                        self.color.bind();
                        gl::VertexAttribPointer(self.attr_colored_color as gl::types::GLuint, 4,
                                                gl::FLOAT, false as gl::types::GLboolean,
                                                0, ptr::null());
                    }
                },
                DrawState::Textured => {
                    self.textured.use_program();

                    unsafe {
                        gl::EnableVertexAttribArray(self.attr_textured_uv as gl::types::GLuint);
                        gl::EnableVertexAttribArray(self.attr_textured_vertex as gl::types::GLuint);
                        gl::EnableVertexAttribArray(self.attr_textured_color as gl::types::GLuint);

                        gl::ActiveTexture(gl::TEXTURE0);

                        self.uv.bind();
                        gl::VertexAttribPointer(self.attr_textured_uv as gl::types::GLuint, 2,
                                                gl::FLOAT, false as gl::types::GLboolean,
                                                0, ptr::null());

                        self.vertex.bind();
                        gl::VertexAttribPointer(self.attr_textured_vertex as gl::types::GLuint, 2,
                                                gl::FLOAT, false as gl::types::GLboolean,
                                                0, ptr::null());

                        self.color.bind();
                        gl::VertexAttribPointer(self.attr_textured_color as gl::types::GLuint, 4,
                                                gl::FLOAT, false as gl::types::GLboolean,
                                                0, ptr::null());
                    }
                },
            }

            self.state = target;
        }
    }

    pub fn new() -> Self {
        let events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new()
            .with_title("Leaffront")
            .with_dimensions(1270, 720);
        let context = glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::Latest)
            .with_gl_profile(glutin::GlProfile::Core)
            .with_vsync(true);
        let gl_window = glutin::GlWindow::new(window,
                                              context, &events_loop).unwrap();

        unsafe {
            gl_window.make_current().unwrap();
        }

        let (width, height) = gl_window.get_inner_size().unwrap();

        unsafe {
            gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

            gl::DebugMessageCallback(
                gl_debug_message,
                ptr::null_mut());

            gl::ClearColor(0.0, 1.0, 0.0, 1.0);
            gl::Viewport(0, 0, width as i32, height as i32);

        }

        let vertex_vbo = GLVBO::new();
        let color_vbo = GLVBO::new();
        let uv_vbo = GLVBO::new();

        unsafe {
            // TODO: Fucking unsafe
            let mut ptr = mem::uninitialized();
            gl::GenVertexArrays(1, &mut ptr);
            gl::BindVertexArray(ptr);
        }

        let colored_shader = GLSLShader::create_shader(
            include_bytes!("../res/shaders/color.vert"),
            include_bytes!("../res/shaders/color.frag")).unwrap();

        colored_shader.use_program();
        let attr_colored_vertex = colored_shader.get_attribute("input_vertex");
        let attr_colored_color = colored_shader.get_attribute("input_color");

        let textured_shader = GLSLShader::create_shader(
            include_bytes!("../res/shaders/tex.vert"),
            include_bytes!("../res/shaders/tex.frag")).unwrap();

        textured_shader.use_program();
        let attr_textured_vertex = textured_shader.get_attribute("input_vertex");
        let attr_textured_color = textured_shader.get_attribute("input_color");
        let attr_textured_uv = textured_shader.get_attribute("input_uv");

        GlutinDrawer {
            events_loop,
            gl_window,
            colored : colored_shader,
            textured : textured_shader,
            state : DrawState::None,
            vertex : vertex_vbo,
            attr_colored_vertex,
            attr_textured_vertex,
            color : color_vbo,
            attr_colored_color,
            attr_textured_color,
            uv : uv_vbo,
            attr_textured_uv,
            background : None,
            transition_count : 0,
            calls : 0
        }
    }
}

impl Drawer for GlutinDrawer {
    type NativeTexture = GlTexture;

    fn start(&mut self) {
        self.calls = 0;
        self.transition_count = 0;

        self.state = DrawState::None;

        let (width, height) = self.gl_window.get_inner_size().unwrap();

        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
        }
    }

    /// Ends this frame.
    fn end(&mut self) {
        self.configure_state(DrawState::None);

        self.gl_window.swap_buffers().unwrap();
    }

    /// Clears the framebuffer.
    fn clear(&mut self, transparent : bool) {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // Draw our background here, if required
        if transparent {
            if self.background.is_some() {
                let size = Rect::new(0, 0,
                                         self.get_width() as i32,
                                         self.get_height() as i32);
                let tex = self.background.take();
                let tex = tex.unwrap();
                self.draw_texture_sized(&tex, &size,
                                    &Color::new_3byte(255, 255, 255));
                self.background = Some(tex);
            }
        }
    }

    /// Enables blending of alpha textures. Disabled at end of frame.
    fn enable_blending(&mut self) {
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
    }

    fn convert_native_texture(&mut self, texture: Texture) -> Self::NativeTexture {
        GlTexture::from_texture(&texture)
    }

    /// Returns the width of the screen.
    fn get_width(&self) -> usize {
        let (width, _) = self.gl_window.get_inner_size().unwrap();
        width as usize
    }

    /// Returns the height of the screen.
    fn get_height(&self) -> usize {
        let (_, height) = self.gl_window.get_inner_size().unwrap();
        height as usize
    }

    /// Draws a texture to the screen, with a specified set of vertices to draw to, a UV
    /// to decode the image with, and a color to use as a base.
    fn draw_textured_vertices_colored_uv(&mut self, texture : &Self::NativeTexture, vertices : &[f32],
                                         colors : &[f32], uv : &[f32]) {
        self.configure_state(DrawState::Textured);

        self.color.set_data(colors);
        self.vertex.set_data(vertices);
        self.uv.set_data(uv);

        texture.bind_texture(gl::TEXTURE_2D);

        unsafe {
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, (vertices.len() / 2) as gl::types::GLsizei);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    /// Draws a set of colored vertices to the screen, with a specified color array.
    fn draw_colored_vertices(&mut self, vertices : &[f32], colors : &[f32]) {
        self.configure_state(DrawState::Colored);

        self.vertex.set_data(vertices);
        self.color.set_data(colors);

        unsafe {
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, (vertices.len() / 2) as gl::types::GLsizei)
        }
    }

    /// Uses the specified image as a background. This is provided as several platforms
    /// have ways to accelerate this beyond OpenGL calls.
    fn set_background(&mut self, image: DynamicImage) {
        let image = GlTexture::from_image(&image.to_rgba());
        self.background = Some(image);
    }

    /// Sets the brightness of the screen.
    fn set_brightness(&mut self, _val: u8) -> ::std::io::Result<()> {
        // NOOP
        Ok(())
    }

    fn get_transition_count(&self) -> usize {
        self.transition_count
    }
}

impl VersionInfo for GlutinDrawer {
    fn version() -> String {
        format!("glutin ({})", env!("CARGO_PKG_VERSION"))
    }
}

extern "system"
fn gl_debug_message(_source: u32, _type: u32, _id: u32, _sev: u32,
                    _len: i32, message: *const c_char,
                    _param: *mut c_void) {
    unsafe {
        let s = cstring_to_string(message);
        println!("OpenGL Debug message: {}", s);
    }
}

unsafe fn cstring_to_string(mut cs: *const c_char) -> String {
    let mut v : Vec<u8> = Vec::new();
    while *cs != 0 {
        v.push(*cs as u8);
        cs = cs.offset(1);
    }
    String::from_utf8(v).expect("c-string not utf8")
}
