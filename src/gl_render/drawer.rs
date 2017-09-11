/// Manages an interface for drawing different kinds of images.

use opengles::glesv2 as gl;

use videocore::bcm_host::GraphicsDisplaySize;

use pi::gl_context::Context;

use gl_render::shader::GLSLShader;
use gl_render::vbo::GLVBO;
use gl_render::texture::GlTexture;
use gl_render::pos::Position;

#[derive(Ord, PartialOrd, Eq, PartialEq)]
enum DrawState {
    None,
    Colored,
    Textured
}

pub struct Drawer {
    state : DrawState,

    size : GraphicsDisplaySize,

    colored : GLSLShader,
    textured : GLSLShader,

    // Used by both shaders
    vertex : GLVBO,

    // Used by colored shader
    color : GLVBO,

    // Used by textured shader
    uv : GLVBO,

    context : Context
}

impl Drawer {
    /// Changes shaders, and ensures that GLES is ready to use it.
    fn configure_state(&mut self, target : DrawState) {
        if self.state != target {
            match target {
                DrawState::None => {
                    panic!("Unable to use no draw state!")
                }
                DrawState::Colored => {
                    self.colored.use_program();
                },
                DrawState::Textured => {
                    self.textured.use_program();
                },
            }

            self.state = target;
        }
    }

    /// Starts this frame.
    pub fn start(&mut self) {
        self.size = Context::get_resolution();
        self.state = DrawState::None;

        gl::clear_color(0.0, 1.0, 0.0, 1.0);
        gl::clear(gl::GL_COLOR_BUFFER_BIT);

        gl::enable(gl::GL_BLEND);
        gl::blend_func(gl::GL_SRC_ALPHA, gl::GL_ONE_MINUS_SRC_ALPHA);
    }

    /// Ends this frame.
    pub fn end(&mut self) {
        self.state = DrawState::None;

        gl::use_program(0);
        gl::bind_buffer(gl::GL_ARRAY_BUFFER, 0);

        gl::disable(gl::GL_BLEND);

        if !self.context.swap_buffers() {
            panic!("Failed to swap buffers!");
        }
    }

    /// Draws a texture to the screen, with a specified set of vertices to draw to, and a UV
    /// to decode the image with.
    pub fn draw_textured_vertices_uv(&mut self, texture : &GlTexture, vertices : &[f32], uv : &[f32]) {
        self.configure_state(DrawState::Textured);

        self.vertex.set_data(vertices);
        self.uv.set_data(uv);

        gl::active_texture(gl::GL_TEXTURE_2D);
        texture.bind_texture(gl::GL_TEXTURE_2D);

        gl::draw_arrays(gl::GL_TRIANGLE_FAN, 0, (vertices.len() / 2) as gl::GLsizei);
    }

    /// Draws a texture to the screen, with a specified set of vertices to draw to, and a
    /// default UV.
    pub fn draw_textured_vertices(&mut self, texture : &GlTexture, vertices : &[f32]) {
        self.draw_textured_vertices_uv(texture, vertices, &[
            0.0, 0.0,
            0.0, 1.0,
            1.0, 1.0,
            0.0, 1.0,
            1.0, 0.0,
            1.0, 1.0
        ])
    }

    /// Draws a texture to the screen, with the specified x/y coordinates (relative to screen size),
    ///  and a specified width/height.
    pub fn draw_texture_sized(&mut self, texture : &GlTexture, pos : Position,
                              width : i32, height : i32) {
        // Translate to OpenGL coordinates
        let min_x = (pos.x as f32) / self.size.width as f32 * 2.0 - 1.0;
        let max_x = ((pos.x + width) as f32) / self.size.width as f32 * 2.0 - 1.0;
        let min_y = (pos.y as f32) / self.size.height as f32 * 2.0 - 1.0;
        let max_y = ((pos.y + height) as f32) / self.size.height as f32 * 2.0 - 1.0;

        // Generate vertex data
        // Inverted due to OpenGL perspective
        let vertices = [
            // Vertex 1
            min_x, -min_y,
            min_x, -max_y,
            max_x, -max_y,
            // Vertex 2
            min_x, -max_y,
            max_x, -min_y,
            max_x, -max_y,
        ];

        self.draw_textured_vertices(texture, &vertices)
    }

    /// Draws a texture to the screen, with the specified x/y coordinates (relative to screen size),
    /// and the texture dimensions as width/height.
    pub fn draw_texture(&mut self, texture : &GlTexture, pos : Position) {
        let width = texture.get_width();
        let height = texture.get_height();

        self.draw_texture_sized(texture, pos, width as i32, height as i32)
    }

    /// Returns the width of the screen.
    pub fn get_width(&self) -> usize {
        return self.size.width as usize;
    }

    /// Returns the height of the screen.
    pub fn get_height(&self) -> usize {
        return self.size.height as usize;
    }

    /// Creates a new drawer.
    pub fn new(context : Context) -> Self {
        let size = Context::get_resolution();

        gl::viewport(0, 0, size.width as i32, size.height as i32);

        let vertex_vbo = GLVBO::new();
        let color_vbo = GLVBO::new();
        let uv_vbo = GLVBO::new();

        let colored_shader = GLSLShader::create_shader(
            include_bytes!("../../res/shaders/color.vert"),
            include_bytes!("../../res/shaders/color.frag")).unwrap();

        colored_shader.use_program();

        let input_vertex = colored_shader.get_attribute("input_vertex");
        let input_color = colored_shader.get_attribute("input_color");

        gl::enable_vertex_attrib_array(input_color as gl::GLuint);
        gl::enable_vertex_attrib_array(input_vertex as gl::GLuint);

        // TODO: Check that these bindings persist
        vertex_vbo.bind();
        gl::vertex_attrib_pointer_offset(input_vertex as gl::GLuint, 2,
                                         gl::GL_FLOAT, false, 0, 0);
        color_vbo.bind();
        gl::vertex_attrib_pointer_offset(input_color as gl::GLuint, 4,
                                         gl::GL_FLOAT, false, 0, 0);

        let textured_shader = GLSLShader::create_shader(
            include_bytes!("../../res/shaders/tex.vert"),
            include_bytes!("../../res/shaders/tex.frag")).unwrap();

        // TODO: Check that these bindings persist
        textured_shader.use_program();

        let input_uv = textured_shader.get_attribute("input_uv");
        let input_vertex = textured_shader.get_attribute("input_vertex");

        gl::enable_vertex_attrib_array(input_uv as gl::GLuint);
        gl::enable_vertex_attrib_array(input_vertex as gl::GLuint);

        uv_vbo.bind();
        gl::vertex_attrib_pointer_offset(input_uv as gl::GLuint, 2,
                                         gl::GL_FLOAT, false, 0, 0);

        vertex_vbo.bind();
        gl::vertex_attrib_pointer_offset(input_vertex as gl::GLuint, 2,
                                         gl::GL_FLOAT, false, 0, 0);

        Drawer {
            context,
            size,
            state    : DrawState::None,
            colored  : colored_shader,
            textured : textured_shader,
            vertex   : vertex_vbo,
            color    : color_vbo,
            uv       : uv_vbo
        }
    }
}

