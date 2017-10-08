/// Manages an interface for drawing different kinds of images.

use opengles::glesv2 as gl;

use videocore::bcm_host::GraphicsDisplaySize;

use gl_context::Context;

use shader::GLSLShader;
use vbo::GLVBO;
use texture::GlTexture;

use leaffront_core::pos::Position;
use leaffront_core::pos::Rect;

use leaffront_core::render::Drawer;
use leaffront_core::render::texture::Texture;
use leaffront_core::render::color::Color;

#[derive(Ord, PartialOrd, Eq, PartialEq)]
enum DrawState {
    None,
    Colored,
    Textured
}

pub struct PiDrawer {
    state : DrawState,

    size : GraphicsDisplaySize,

    colored : GLSLShader,
    textured : GLSLShader,

    // Used by both shaders
    vertex : GLVBO,
    attr_colored_vertex : gl::GLint,
    attr_textured_vertex : gl::GLint,

    // Used by colored shader
    color : GLVBO,
    attr_colored_color : gl::GLint,
    attr_textured_color : gl::GLint,

    // Used by textured shader
    uv : GLVBO,
    attr_textured_uv : gl::GLint,

    pub context : Context
}

impl PiDrawer {
    /// Changes shaders, and ensures that GLES is ready to use it.
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

                    self.vertex.bind();
                    gl::vertex_attrib_pointer_offset(self.attr_colored_vertex as gl::GLuint, 2,
                                                     gl::GL_FLOAT, false, 0, 0);

                    self.color.bind();
                    gl::vertex_attrib_pointer_offset(self.attr_colored_color as gl::GLuint, 4,
                                                     gl::GL_FLOAT, false, 0, 0);
                },
                DrawState::Textured => {
                    self.textured.use_program();
                    gl::active_texture(gl::GL_TEXTURE_2D);

                    self.uv.bind();
                    gl::vertex_attrib_pointer_offset(self.attr_textured_uv as gl::GLuint, 2,
                                                     gl::GL_FLOAT, false, 0, 0);

                    self.vertex.bind();
                    gl::vertex_attrib_pointer_offset(self.attr_textured_vertex as gl::GLuint, 2,
                                                     gl::GL_FLOAT, false, 0, 0);

                    self.color.bind();
                    gl::vertex_attrib_pointer_offset(self.attr_textured_color as gl::GLuint, 4,
                                                     gl::GL_FLOAT, false, 0, 0);
                },
            }

            self.state = target;
        }
    }

    /// Creates a new drawer.
    pub fn new() -> Self {
        let context = Context::build().unwrap();

        let size = Context::get_resolution();

        gl::viewport(0, 0, size.width as i32, size.height as i32);

        let vertex_vbo = GLVBO::new();
        let color_vbo = GLVBO::new();
        let uv_vbo = GLVBO::new();

        let colored_shader = GLSLShader::create_shader(
            include_bytes!("../../res/shaders/color.vert"),
            include_bytes!("../../res/shaders/color.frag")).unwrap();

        colored_shader.use_program();
        let attr_colored_vertex = colored_shader.get_attribute("input_vertex");
        let attr_colored_color = colored_shader.get_attribute("input_color");

        gl::enable_vertex_attrib_array(attr_colored_color as gl::GLuint);
        gl::enable_vertex_attrib_array(attr_colored_vertex as gl::GLuint);

        let textured_shader = GLSLShader::create_shader(
            include_bytes!("../../res/shaders/tex.vert"),
            include_bytes!("../../res/shaders/tex.frag")).unwrap();

        textured_shader.use_program();
        let attr_textured_vertex = textured_shader.get_attribute("input_vertex");
        let attr_textured_color = textured_shader.get_attribute("input_color");
        let attr_textured_uv = textured_shader.get_attribute("input_uv");

        gl::enable_vertex_attrib_array(attr_textured_color as gl::GLuint);
        gl::enable_vertex_attrib_array(attr_textured_uv as gl::GLuint);
        gl::enable_vertex_attrib_array(attr_textured_vertex as gl::GLuint);

        Self {
            context,
            size,
            state    : DrawState::None,
            colored  : colored_shader,
            textured : textured_shader,
            vertex   : vertex_vbo,
            attr_colored_vertex,
            attr_textured_vertex,
            color    : color_vbo,
            attr_colored_color,
            attr_textured_color,
            uv       : uv_vbo,
            attr_textured_uv
        }
    }
}

impl Drawer for PiDrawer {
    type NativeTexture = GlTexture;

    fn start(&mut self) {
        self.size = Context::get_resolution();
        self.state = DrawState::None;
    }

    /// Ends this frame.
    fn end(&mut self) {
        self.state = DrawState::None;

        gl::use_program(0);
        gl::bind_buffer(gl::GL_ARRAY_BUFFER, 0);

        gl::disable(gl::GL_BLEND);

        if !self.context.swap_buffers() {
            panic!("Failed to swap buffers!");
        }
    }

    /// Clears the framebuffer.
    fn clear(&mut self, transparent : bool) {
        if transparent {
            gl::clear_color(0.0, 0.0, 0.0, 0.0);
        } else {
            gl::clear_color(0.0, 0.0, 0.0, 1.0);
        }
        gl::clear(gl::GL_COLOR_BUFFER_BIT);
    }

    /// Enables blending of alpha textures. Disabled at end of frame.
    fn enable_blending(&mut self) {
        gl::enable(gl::GL_BLEND);
        gl::blend_func(gl::GL_SRC_ALPHA, gl::GL_ONE_MINUS_SRC_ALPHA);
    }

    fn convert_native_texture(&mut self, texture: Texture) -> Self::NativeTexture {
        GlTexture::from_texture(&texture)
    }

    /// Returns the width of the screen.
    fn get_width(&self) -> usize {
        return self.size.width as usize;
    }

    /// Returns the height of the screen.
    fn get_height(&self) -> usize {
        return self.size.height as usize;
    }


    /// Draws a texture to the screen, with a specified set of vertices to draw to, a UV
    /// to decode the image with, and a color to use as a base.
    fn draw_textured_vertices_colored_uv(&mut self, texture : &Self::NativeTexture, vertices : &[f32],
                                             colors : &[f32], uv : &[f32]) {
        self.configure_state(DrawState::Textured);

        self.color.set_data(colors);
        self.vertex.set_data(vertices);
        self.uv.set_data(uv);

        texture.bind_texture(gl::GL_TEXTURE_2D);

        gl::draw_arrays(gl::GL_TRIANGLE_STRIP, 0, (vertices.len() / 2) as gl::GLsizei);
    }

    /// Draws a set of colored vertices to the screen, with a specified color array.
    fn draw_colored_vertices(&mut self, vertices : &[f32], colors : &[f32]) {
        self.configure_state(DrawState::Colored);

        self.vertex.set_data(vertices);
        self.color.set_data(colors);

        gl::draw_arrays(gl::GL_TRIANGLE_STRIP, 0, (vertices.len() / 2) as gl::GLsizei);
    }

}
