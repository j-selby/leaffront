use gl;

use std::mem;

pub struct GLVBO {
    ptr : gl::types::GLuint
}

impl GLVBO {
    /// Binds this buffer.
    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.ptr)
        }
    }

    /// Sets the data within this VBO. Implicitly binds the buffer.
    pub fn set_data<T>(&self, data : &[T]) {
        self.bind();
        unsafe {
            gl::BufferData(gl::ARRAY_BUFFER, (data.len() * mem::size_of::<T>()) as gl::types::GLsizeiptr,
                           data.as_ptr() as *const _, gl::STATIC_DRAW)
        }
    }

    /// Creates a new OpenGL VBO.
    pub fn new() -> Self {
        let mut ptr = unsafe { mem::uninitialized() };

        unsafe {
            gl::GenBuffers(1, &mut ptr);
        }

        GLVBO {
            ptr
        }
    }
}

impl Drop for GLVBO {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, [self.ptr].as_ptr())
        }
    }
}
