use gl;

use std::mem;
use std::mem::MaybeUninit;

pub struct GLVBO {
    ptr: MaybeUninit<gl::types::GLuint>,
}

impl GLVBO {
    /// Binds this buffer.
    pub fn bind(&self) {
        unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, self.ptr.assume_init()) }
    }

    /// Sets the data within this VBO. Implicitly binds the buffer.
    pub fn set_data<T>(&self, data: &[T]) {
        self.bind();
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (data.len() * mem::size_of::<T>()) as gl::types::GLsizeiptr,
                data.as_ptr() as *const _,
                gl::STATIC_DRAW,
            )
        }
    }

    /// Creates a new OpenGL VBO.
    pub fn new() -> Self {
        let mut ptr = MaybeUninit::uninit();

        unsafe {
            gl::GenBuffers(1, ptr.as_mut_ptr());
        }

        GLVBO { ptr }
    }
}

impl Drop for GLVBO {
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, [self.ptr.assume_init()].as_ptr()) }
    }
}
