/// Manages an interface for drawing different kinds of images.

use opengles::glesv2 as gl;

use gl_render::shader::GLSLShader;
use gl_render::vbo::GLVBO;
use gl_render::texture::GlTexture;

#[derive(Ord, PartialOrd, Eq, PartialEq)]
enum DrawState {
    None,
    Colored,
    Textured
}

pub struct Drawer {
    state : DrawState,

    colored : GLSLShader,
    textured : GLSLShader,

    // Used by both shaders
    vertex : GLVBO,

    // Used by colored shader
    color : GLVBO,

    // Used by textured shader
    uv : GLVBO
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

    /// Draws a texture to the screen, with a specified set of vertices to draw to, and a UV
    /// to decode the image with.
    pub fn draw_textured_vertices_uv(&mut self, texture : GlTexture, vertices : &[f32], uv : &[f32]) {
        self.configure_state(DrawState::Textured);

        self.vertex.set_data(vertices);
        self.uv.set_data(uv);

        gl::active_texture(gl::GL_TEXTURE_2D);
        texture.bind_texture(gl::GL_TEXTURE_2D);

        gl::draw_arrays(gl::GL_TRIANGLE_FAN, 0, (vertices.len() / 2) as gl::GLsizei);
    }

    /// Draws a texture to the screen, with a specified set of vertices to draw to, and a
    /// default UV.
    pub fn draw_textured_vertices(&mut self, texture : GlTexture, vertices : &[f32]) {
        self.draw_textured_vertices_uv(texture, vertices, &[
            0.0, 0.0,
            0.0, 1.0,
            1.0, 1.0,
            0.0, 1.0,
            1.0, 0.0,
            1.0, 1.0
        ])
    }

    /// Creates a new drawer.
    pub fn new() -> Self {
        let colored_shader = GLSLShader::create_shader(
            include_bytes!("../../res/shaders/color.vert"),
            include_bytes!("../../res/shaders/color.frag")).unwrap();
        let textured_shader = GLSLShader::create_shader(
            include_bytes!("../../res/shaders/tex.vert"),
            include_bytes!("../../res/shaders/tex.frag")).unwrap();

        Drawer {
            state    : DrawState::None,
            colored  : colored_shader,
            textured : textured_shader,
            vertex   : GLVBO::new(),
            color    : GLVBO::new(),
            uv       : GLVBO::new()
        }
    }
}

