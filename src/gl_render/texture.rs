/// A holder for a OpenGL texture

use opengles::glesv2 as gl;

use texture::Texture;

use image::RgbaImage;

pub struct GlTexture {
    width  : usize,
    height : usize,
    ptr    : gl::GLuint
}

impl GlTexture {
    pub fn get_raw_ptr(&self) -> gl::GLuint {
        return self.ptr;
    }

    /// Converts a RGBA byte array to a OpenGL reference.
    pub fn from_bytes(bytes : &[u8], width : usize, height : usize) -> Self {
        let texture_ref : gl::GLuint = gl::gen_textures(1)[0];
        gl::bind_texture(gl::GL_TEXTURE_2D, texture_ref);
        gl::tex_image_2d(gl::GL_TEXTURE_2D, 0 as gl::GLint, gl::GL_RGBA as gl::GLint,
                         width as gl::GLint, height as gl::GLint, 0,
                         gl::GL_RGBA, gl::GL_UNSIGNED_BYTE,
                         &bytes);

        gl::tex_parameteri(gl::GL_TEXTURE_2D, gl::GL_TEXTURE_MIN_FILTER,
                           gl::GL_LINEAR as gl::GLint);
        gl::tex_parameteri(gl::GL_TEXTURE_2D, gl::GL_TEXTURE_MAG_FILTER,
                           gl::GL_LINEAR as gl::GLint);

        gl::tex_parameteri(gl::GL_TEXTURE_2D, gl::GL_TEXTURE_WRAP_S,
                           gl::GL_CLAMP_TO_EDGE as gl::GLint);
        gl::tex_parameteri(gl::GL_TEXTURE_2D, gl::GL_TEXTURE_WRAP_T,
                           gl::GL_CLAMP_TO_EDGE as gl::GLint);

        gl::generate_mipmap(gl::GL_TEXTURE_2D);

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
}

impl Drop for GlTexture {
    fn drop(&mut self) {
        gl::delete_textures(&[self.ptr]);
    }
}
