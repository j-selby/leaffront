/// Manages an interface for drawing different kinds of images.
use opengles::glesv2 as gl;

use image::DynamicImage;

use videocore::dispmanx;
use videocore::dispmanx::ResourceHandle;
use videocore::dispmanx::Transform;
use videocore::image::ImageType;
use videocore::image::Rect as VCRect;

use libc::c_void;

use videocore::bcm_host::GraphicsDisplaySize;

use gl_context::Context;

use brightness::set_brightness;

use shader::GLSLShader;
use texture::GlTexture;
use vbo::GLVBO;

use leaffront_core::pos::Rect;
use leaffront_core::render::texture::Texture;
use leaffront_core::render::Drawer;
use leaffront_core::version::VersionInfo;

#[derive(Ord, PartialOrd, Eq, PartialEq)]
enum DrawState {
    None,
    Colored,
    Textured,
}

pub struct PiDrawer {
    state: DrawState,

    size: GraphicsDisplaySize,

    colored: GLSLShader,
    textured: GLSLShader,

    // Used by both shaders
    vertex: GLVBO,
    attr_colored_vertex: gl::GLint,
    attr_textured_vertex: gl::GLint,

    // Used by colored shader
    color: GLVBO,
    attr_colored_color: gl::GLint,
    attr_textured_color: gl::GLint,

    // Used by textured shader
    uv: GLVBO,
    attr_textured_uv: gl::GLint,

    context: Context,

    bg: Option<ResourceHandle>,

    // Debugging information
    transitions: usize,
}

impl PiDrawer {
    /// Changes shaders, and ensures that GLES is ready to use it.
    fn configure_state(&mut self, target: DrawState) {
        if self.state != target {
            self.transitions += 1;

            // TODO: Unbind previous state, if needed
            // (disable_vertex_attrib_array)
            match self.state {
                DrawState::None => {}
                DrawState::Colored => {
                    gl::disable_vertex_attrib_array(self.attr_colored_vertex as gl::GLuint);
                    gl::disable_vertex_attrib_array(self.attr_colored_color as gl::GLuint);
                }
                DrawState::Textured => {
                    gl::disable_vertex_attrib_array(self.attr_textured_uv as gl::GLuint);
                    gl::disable_vertex_attrib_array(self.attr_textured_vertex as gl::GLuint);
                    gl::disable_vertex_attrib_array(self.attr_textured_color as gl::GLuint);
                }
            }

            // Configure new state
            match target {
                DrawState::None => {
                    gl::use_program(0);
                    gl::bind_buffer(gl::GL_ARRAY_BUFFER, 0);
                }
                DrawState::Colored => {
                    self.colored.use_program();

                    gl::enable_vertex_attrib_array(self.attr_colored_color as gl::GLuint);
                    gl::enable_vertex_attrib_array(self.attr_colored_vertex as gl::GLuint);

                    self.vertex.bind();
                    gl::vertex_attrib_pointer_offset(
                        self.attr_colored_vertex as gl::GLuint,
                        2,
                        gl::GL_FLOAT,
                        false,
                        0,
                        0,
                    );

                    self.color.bind();
                    gl::vertex_attrib_pointer_offset(
                        self.attr_colored_color as gl::GLuint,
                        4,
                        gl::GL_FLOAT,
                        false,
                        0,
                        0,
                    );
                }
                DrawState::Textured => {
                    self.textured.use_program();

                    gl::enable_vertex_attrib_array(self.attr_textured_color as gl::GLuint);
                    gl::enable_vertex_attrib_array(self.attr_textured_uv as gl::GLuint);
                    gl::enable_vertex_attrib_array(self.attr_textured_vertex as gl::GLuint);

                    gl::active_texture(gl::GL_TEXTURE0);

                    self.uv.bind();
                    gl::vertex_attrib_pointer_offset(
                        self.attr_textured_uv as gl::GLuint,
                        2,
                        gl::GL_FLOAT,
                        false,
                        0,
                        0,
                    );

                    self.vertex.bind();
                    gl::vertex_attrib_pointer_offset(
                        self.attr_textured_vertex as gl::GLuint,
                        2,
                        gl::GL_FLOAT,
                        false,
                        0,
                        0,
                    );

                    self.color.bind();
                    gl::vertex_attrib_pointer_offset(
                        self.attr_textured_color as gl::GLuint,
                        4,
                        gl::GL_FLOAT,
                        false,
                        0,
                        0,
                    );
                }
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
            include_bytes!("../res/shaders/color.vert"),
            include_bytes!("../res/shaders/color.frag"),
        )
        .unwrap();

        colored_shader.use_program();
        let attr_colored_vertex = colored_shader.get_attribute("input_vertex");
        let attr_colored_color = colored_shader.get_attribute("input_color");

        let textured_shader = GLSLShader::create_shader(
            include_bytes!("../res/shaders/tex.vert"),
            include_bytes!("../res/shaders/tex.frag"),
        )
        .unwrap();

        textured_shader.use_program();
        let attr_textured_vertex = textured_shader.get_attribute("input_vertex");
        let attr_textured_color = textured_shader.get_attribute("input_color");
        let attr_textured_uv = textured_shader.get_attribute("input_uv");

        Self {
            context,
            size,
            state: DrawState::None,
            colored: colored_shader,
            textured: textured_shader,
            vertex: vertex_vbo,
            attr_colored_vertex,
            attr_textured_vertex,
            color: color_vbo,
            attr_colored_color,
            attr_textured_color,
            uv: uv_vbo,
            attr_textured_uv,
            bg: None,
            transitions: 0,
        }
    }
}

impl Drawer for PiDrawer {
    type NativeTexture = GlTexture;

    fn start(&mut self) {
        self.transitions = 0;
        self.size = Context::get_resolution();
        self.state = DrawState::None;
    }

    /// Ends this frame.
    fn end(&mut self) {
        self.configure_state(DrawState::None);

        gl::disable(gl::GL_BLEND);

        if !self.context.swap_buffers() {
            panic!("Failed to swap buffers!");
        }
    }

    /// Clears the framebuffer.
    fn clear(&mut self, transparent: bool) {
        if transparent {
            gl::clear_color(0.0, 0.0, 0.0, 0.0);
        } else {
            gl::clear_color(0.0, 0.0, 0.0, 1.0);
        }
        gl::clear(gl::GL_COLOR_BUFFER_BIT);
    }

    /// Enables blending of alpha textures. Disabled at end of frame.
    fn enable_blending(&mut self) {
        gl::disable(gl::GL_CULL_FACE);
        gl::enable(gl::GL_SCISSOR_TEST);
        gl::enable(gl::GL_BLEND);
        gl::blend_func(gl::GL_ONE, gl::GL_ONE_MINUS_SRC_ALPHA);
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
    fn draw_textured_vertices_colored_uv(
        &mut self,
        texture: &Self::NativeTexture,
        vertices: &[f32],
        colors: &[f32],
        uv: &[f32],
    ) {
        self.configure_state(DrawState::Textured);

        self.color.set_data(colors);
        self.vertex.set_data(vertices);
        self.uv.set_data(uv);

        texture.bind_texture(gl::GL_TEXTURE_2D);

        gl::draw_arrays(gl::GL_TRIANGLES, 0, (vertices.len() / 2) as gl::GLsizei);
    }

    /// Draws a set of colored vertices to the screen, with a specified color array.
    fn draw_colored_vertices(&mut self, vertices: &[f32], colors: &[f32]) {
        self.configure_state(DrawState::Colored);

        self.vertex.set_data(vertices);
        self.color.set_data(colors);

        gl::draw_arrays(
            gl::GL_TRIANGLE_STRIP,
            0,
            (vertices.len() / 2) as gl::GLsizei,
        );
    }

    /// Uses the specified image as a background. This is provided as several platforms
    /// have ways to accelerate this beyond OpenGL calls.
    fn set_background(&mut self, image: DynamicImage) {
        match self.bg {
            Some(resource) => {
                dispmanx::resource_delete(resource);
            }
            _ => {}
        }

        let bg_img = image.to_rgb8();

        // Resize the background to the correct size
        //let size = Context::get_resolution();

        // Pad out the image, if required
        let target_width;
        let target_height;
        let padding;

        let mut img_buffer = if bg_img.width() % 16 != 0 {
            // Find the next multiple that *is* even
            padding = 16 - (bg_img.width() % 16);
            target_width = bg_img.width();
            target_height = bg_img.height();

            let old_width = bg_img.width();

            let bg_img = bg_img.to_vec();

            let mut buf: Vec<u8> = vec![0; ((target_width + padding) * target_height * 3) as usize];
            for y in 0..target_height {
                for x in 0..old_width {
                    buf[((y * (target_width + padding) + x) * 3) as usize] =
                        bg_img[((y * old_width + x) * 3) as usize];
                    buf[((y * (target_width + padding) + x) * 3 + 1) as usize] =
                        bg_img[((y * old_width + x) * 3 + 1) as usize];
                    buf[((y * (target_width + padding) + x) * 3 + 2) as usize] =
                        bg_img[((y * old_width + x) * 3 + 2) as usize];
                }
            }

            buf
        } else {
            target_width = bg_img.width();
            target_height = bg_img.height();
            padding = 0;
            bg_img.to_vec()
        };

        let bg_ptr = img_buffer.as_mut_ptr() as *mut _ as *mut c_void;
        let mut ptr = 0; // Unused

        let dest_rect = VCRect {
            x: 0,
            y: 0,
            width: target_width as i32,
            height: target_height as i32,
        };

        let element = self.context.bg_element;

        let bg_resource = dispmanx::resource_create(
            ImageType::RGB888,
            target_width as u32,
            target_height as u32,
            &mut ptr,
        );

        if dispmanx::resource_write_data(
            bg_resource,
            ImageType::RGB888,
            (3 * (target_width + padding)) as i32,
            bg_ptr,
            &dest_rect,
        ) {
            warn!("Failed to write data")
        }

        let update = dispmanx::update_start(10);

        // Resize the element's src attr
        let src_rect = VCRect {
            x: 0,
            y: 0,
            width: (target_width as i32) << 16,
            height: (target_height as i32) << 16,
        };

        dispmanx::element_change_attributes(
            update,
            element,
            (1 << 3) | (1 << 2),
            0,                    // Ignored
            255,                  // Ignored
            &src_rect,            //&dest_rect,
            &dest_rect,           //&src_rect,
            0,                    // Ignored
            Transform::NO_ROTATE, // Ignored
        );

        if dispmanx::element_change_source(update, element, bg_resource) {
            warn!("Resource change failed!");
        }

        if dispmanx::update_submit_sync(update) {
            warn!("Failed to update");
        }

        self.bg = Some(bg_resource);
    }

    /// Sets the brightness of the screen.
    fn set_brightness(&mut self, val: u8) -> ::std::io::Result<()> {
        leaffront_core::brightness::set_brightness(val)
    }

    fn set_fullscreen(&mut self, _fullscreen: bool) {
        // NOOP
    }

    fn get_transition_count(&self) -> usize {
        self.transitions
    }

    fn start_clip(&self, rect: &Rect) {
        let min_x = rect.x;
        let min_y = rect.y;
        let max_x = rect.x + rect.width;
        let max_y = rect.y + rect.height;

        gl::scissor(
            min_x as _,
            self.get_height() as i32 - max_y as i32,
            (max_x - min_x) as _,
            (max_y - min_y) as _,
        );
    }

    fn end_clip(&self) {
        gl::disable(gl::GL_SCISSOR_TEST);
    }
}

impl VersionInfo for PiDrawer {
    fn version() -> String {
        format!("opengles + dispmanx ({})", env!("CARGO_PKG_VERSION"))
    }
}

impl Drop for PiDrawer {
    fn drop(&mut self) {
        debug!("Cleaning up background: ");

        match self.bg {
            Some(resource) => {
                dispmanx::resource_delete(resource);
            }
            _ => {}
        }

        debug!("Done!");
    }
}
