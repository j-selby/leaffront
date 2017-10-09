use leaffront_core::render::Drawer;
use leaffront_core::render::texture::Texture;

use image::DynamicImage;

use texture::GlTexture;

use shader::GLSLShader;

use vbo::GLVBO;

use glutin;
use glutin::GlContext;

use gl;

use libc;

use std::ptr;
use std::mem::transmute;

use std::os::raw::c_void;
use std::os::raw::c_char;

#[derive(Ord, PartialOrd, Eq, PartialEq)]
enum DrawState {
    None,
    Colored,
    Textured
}

pub struct GlutinDrawer {
    events_loop : glutin::EventsLoop,
    gl_window : glutin::GlWindow,

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

    state : DrawState,
}

impl GlutinDrawer {
    /// Changes shaders, and ensures that G is ready to use it.
    fn configure_state(&mut self, target : DrawState) {
        if self.state != target {
            // TODO: Unbind previous state, if needed
            // (disable_vertex_attrib_array)

            match target {
                DrawState::None => {
                    panic!("Unable to use no draw state!")
                }
                DrawState::Colored => {
                    self.colored.use_program();

                    unsafe {
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
                        gl::ActiveTexture(gl::TEXTURE_2D);

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
        let mut events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new()
            .with_title("Leaffront")
            .with_dimensions(1270, 720);
        let context = glutin::ContextBuilder::new()
            .with_gl_debug_flag(true)
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

        let colored_shader = GLSLShader::create_shader(
            include_bytes!("../res/shaders/color.vert"),
            include_bytes!("../res/shaders/color.frag")).unwrap();

        colored_shader.use_program();
        let attr_colored_vertex = colored_shader.get_attribute("input_vertex");
        let attr_colored_color = colored_shader.get_attribute("input_color");

        unsafe {
            gl::EnableVertexAttribArray(attr_colored_color as gl::types::GLuint);
            gl::EnableVertexAttribArray(attr_colored_vertex as gl::types::GLuint);
        }

        let textured_shader = GLSLShader::create_shader(
            include_bytes!("../res/shaders/tex.vert"),
            include_bytes!("../res/shaders/tex.frag")).unwrap();

        textured_shader.use_program();
        let attr_textured_vertex = textured_shader.get_attribute("input_vertex");
        let attr_textured_color = textured_shader.get_attribute("input_color");
        let attr_textured_uv = textured_shader.get_attribute("input_uv");

        unsafe {
            gl::EnableVertexAttribArray(attr_textured_color as gl::types::GLuint);
            gl::EnableVertexAttribArray(attr_textured_uv as gl::types::GLuint);
            gl::EnableVertexAttribArray(attr_textured_vertex as gl::types::GLuint);
        }

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
        }
    }
}

impl Drawer for GlutinDrawer {
    type NativeTexture = GlTexture;

    fn start(&mut self) {
        self.state = DrawState::None;
    }

    /// Ends this frame.
    fn end(&mut self) {
        self.gl_window.swap_buffers().unwrap();
    }

    /// Clears the framebuffer.
    fn clear(&mut self, transparent : bool) {
        unsafe {
            if transparent {
                gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            } else {
                gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            }
            gl::Clear(gl::COLOR_BUFFER_BIT);
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
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, (vertices.len() / 2) as gl::types::GLsizei)
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
        unimplemented!();
    }

    /// Sets the brightness of the screen.
    fn set_brightness(&mut self, _val: u8) -> ::std::io::Result<()> {
        // NOOP
        Ok(())
    }
}

extern "system"
fn gl_debug_message(_source: u32, _type: u32, _id: u32, _sev: u32,
                    _len: i32, message: *const c_char,
                    _param: *mut c_void)
{
    unsafe {
        let s = cstring_to_string(message);
        panic!("OpenGL Debug message: {}", s);
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
