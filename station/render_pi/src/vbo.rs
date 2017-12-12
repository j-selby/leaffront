/// Manages OpenGLES VBOs.

use opengles::glesv2 as gl;

pub struct GLVBO {
    ptr : gl::GLuint
}

impl GLVBO {
    /// Binds this buffer.
    pub fn bind(&self) {
        gl::bind_buffer(gl::GL_ARRAY_BUFFER, self.ptr);
    }

    /// Sets the data within this VBO. Implicitly binds the buffer.
    pub fn set_data<T>(&self, data : &[T]) {
        self.bind();
        gl::buffer_data(gl::GL_ARRAY_BUFFER, data, gl::GL_STATIC_DRAW)
    }

    /// Creates a new OpenGL VBO.
    pub fn new() -> Self {
        GLVBO {
            ptr : gl::gen_buffers(1)[0]
        }
    }
}

impl Drop for GLVBO {
    fn drop(&mut self) {
        gl::delete_buffers(&[self.ptr])
    }
}
