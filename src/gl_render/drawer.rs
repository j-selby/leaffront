/// Manages an interface for drawing different kinds of images.

use opengles::glesv2 as gl;

use videocore::bcm_host::GraphicsDisplaySize;

use pi::gl_context::Context;

use gl_render::shader::GLSLShader;
use gl_render::vbo::GLVBO;
use gl_render::texture::GlTexture;
use gl_render::pos::Position;
use gl_render::pos::Rect;

use color::Color;

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

impl Drawer {
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

    fn rect_to_vertices(&self, rect : &Rect) -> [f32; 12] {
        // Translate to OpenGL coordinates
        let min_x = (rect.x as f32) / self.size.width as f32 * 2.0 - 1.0;
        let max_x = ((rect.x + rect.width) as f32) / self.size.width as f32 * 2.0 - 1.0;
        let min_y = (rect.y as f32) / self.size.height as f32 * 2.0 - 1.0;
        let max_y = ((rect.y + rect.height) as f32) / self.size.height as f32 * 2.0 - 1.0;

        // Generate vertex data
        // Inverted due to OpenGL perspective
        [
            // Vertex 1
            min_x, -min_y,
            min_x, -max_y,
            max_x, -max_y,
            // Vertex 2
            min_x, -min_y,
            max_x, -min_y,
            max_x, -max_y,
        ]
    }

    /// Starts this frame.
    pub fn start(&mut self) {
        self.size = Context::get_resolution();
        self.state = DrawState::None;

        gl::clear_color(0.0, 0.0, 0.0, 0.0);
        gl::clear(gl::GL_COLOR_BUFFER_BIT);
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

    /// Waits for vertical sync.
    pub fn vsync(&self) {
        self.context.wait_for_vsync();
    }

    /// Enables blending of alpha textures. Disabled at end of frame.
    pub fn enable_blending(&self) {
        gl::enable(gl::GL_BLEND);
        gl::blend_func(gl::GL_SRC_ALPHA, gl::GL_ONE_MINUS_SRC_ALPHA);
    }

    /// Draws a texture to the screen, with a specified set of vertices to draw to, a UV
    /// to decode the image with, and a color to use as a base.
    pub fn draw_textured_vertices_colored_uv(&mut self, texture : &GlTexture, vertices : &[f32],
                                             colors : &[f32], uv : &[f32]) {
        self.configure_state(DrawState::Textured);

        self.color.set_data(colors);
        self.vertex.set_data(vertices);
        self.uv.set_data(uv);

        texture.bind_texture(gl::GL_TEXTURE_2D);

        gl::draw_arrays(gl::GL_TRIANGLE_STRIP, 0, (vertices.len() / 2) as gl::GLsizei);
    }

    /// Draws a texture to the screen, with a specified set of vertices to draw to, and a color
    /// to use as a base.
    pub fn draw_textured_vertices_colored(&mut self, texture : &GlTexture, vertices : &[f32],
                                          colors : &[f32]) {
        self.draw_textured_vertices_colored_uv(texture, vertices, colors, &[
            0.0, 0.0,
            0.0, 1.0,
            1.0, 1.0,
            0.0, 0.0,
            1.0, 0.0,
            1.0, 1.0
        ])
    }

    /// Draws a texture to the screen, with a specified set of vertices to draw to, and a
    /// default UV.
    pub fn draw_textured_vertices(&mut self, texture : &GlTexture, vertices : &[f32]) {
        self.draw_textured_vertices_colored(texture, vertices, &[1.0; 24])
    }

    /// Draws a texture to the screen, with the specified x/y coordinates (relative to screen size),
    ///  and a specified width/height.
    pub fn draw_texture_sized(&mut self, texture : &GlTexture, rect : &Rect, color : &Color) {
        let vertices = self.rect_to_vertices(rect);

        let mut colors : [f32; 24] = [0.0; 24];

        for i in 0 .. 24 / 4 {
            colors[i * 4] = color.r as f32 / 255.0;
            colors[i * 4 + 1] = color.g as f32 / 255.0;
            colors[i * 4 + 2] = color.b as f32 / 255.0;
            colors[i * 4 + 3] = color.a as f32 / 255.0;
        }

        self.draw_textured_vertices_colored(texture, &vertices, &colors)
    }

    /// Draws a texture to the screen, with the specified x/y coordinates (relative to screen size),
    /// and the texture dimensions as width/height.
    pub fn draw_texture_colored(&mut self, texture : &GlTexture, pos : &Position, color : &Color) {
        let width = texture.get_width();
        let height = texture.get_height();

        self.draw_texture_sized(texture, &Rect::new_from_pos(pos,
                                                             width as i32, height as i32), color)
    }

    /// Draws a texture to the screen, with the specified x/y coordinates (relative to screen size),
    /// and the texture dimensions as width/height.
    pub fn draw_texture(&mut self, texture : &GlTexture, pos : &Position) {
        // TODO: Potentially dedicated shader for non colored?
        self.draw_texture_colored(texture, pos, &Color::new_4byte(255, 255, 255, 255))
    }

    /// Draws a set of colored vertices to the screen, with a specified color array.
    pub fn draw_colored_vertices(&mut self, vertices : &[f32], colors : &[f32]) {
        self.configure_state(DrawState::Colored);

        self.vertex.set_data(vertices);
        self.color.set_data(colors);

        gl::draw_arrays(gl::GL_TRIANGLE_STRIP, 0, (vertices.len() / 2) as gl::GLsizei);
    }

    /// Draws a colored rectangle to the screen, with a single color.
    pub fn draw_colored_rect(&mut self, rect : &Rect, color : &Color) {
        let vertices : [f32; 12] = self.rect_to_vertices(&rect);
        let mut colors : [f32; 24] = [0.0; 24];

        for i in 0 .. 24 / 4 {
            colors[i * 4] = color.r as f32 / 255.0;
            colors[i * 4 + 1] = color.g as f32 / 255.0;
            colors[i * 4 + 2] = color.b as f32 / 255.0;
            colors[i * 4 + 3] = color.a as f32 / 255.0;
        }

        self.draw_colored_vertices(&vertices, &colors)
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

        Drawer {
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

