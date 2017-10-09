use leaffront_core::render::Dimensions;
use leaffront_core::render::texture::Texture;

use gl;

use std::mem;

use image::RgbaImage;

pub struct GlTexture {
    width  : usize,
    height : usize,
    ptr    : gl::types::GLuint
}

impl GlTexture {
    /// Converts a RGBA byte array to a OpenGL reference.
    fn from_bytes(bytes : &[u8], width : usize, height : usize) -> Self {
        let mut texture_ref = unsafe { mem::uninitialized() };
        unsafe {
            gl::GenTextures(1, &mut texture_ref);
            gl::BindTexture(gl::TEXTURE_2D, texture_ref);
            gl::TexImage2D(gl::TEXTURE_2D, 0 as gl::types::GLint, gl::RGBA as gl::types::GLint,
                             width as gl::types::GLint, height as gl::types::GLint, 0,
                             gl::RGBA, gl::UNSIGNED_BYTE,
                             bytes.as_ptr() as *const _);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER,
                               gl::LINEAR as gl::types::GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER,
                               gl::LINEAR as gl::types::GLint);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S,
                               gl::CLAMP_TO_EDGE as gl::types::GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T,
                               gl::CLAMP_TO_EDGE as gl::types::GLint);

            gl::GenerateMipmap(gl::TEXTURE_2D);
        }

        return GlTexture {
            width,
            height,
            ptr : texture_ref
        };
    }

    /// Converts a texture to a OpenGL reference.
    pub fn from_texture(tex : &Texture) -> Self {
        GlTexture::from_bytes(&tex.tex_data, tex.get_width(), tex.get_height())
    }

    /// Converts a image to a OpenGL reference.
    pub fn from_image(tex : &RgbaImage) -> Self {
        GlTexture::from_bytes(tex.as_ref(), tex.width() as usize, tex.height() as usize)
    }

    /// Binds this OpenGL texture. This struct must
    /// remain in scope for the entire duration of usage.
    pub fn bind_texture(&self, target : gl::types::GLenum) {
        unsafe {
            gl::BindTexture(target, self.ptr)
        }
    }
}

impl Dimensions for GlTexture {
    /// Returns the width of this texture.
    fn get_width(&self) -> usize {
        self.width
    }

    /// Returns the height of this texture.
    fn get_height(&self) -> usize {
        self.height
    }
}
