/// Holds and parses GLSL shaders.

use gl;
use gl::types::GLint;

use std::ptr;

use std::ffi::CString;

pub struct GLSLShader {
    program : gl::types::GLuint,
    vertex : gl::types::GLuint,
    fragment : gl::types::GLuint,
}

impl GLSLShader {
    /// Enables this program to be used.
    /// Shader MUST remain in scope for duration of usage.
    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.program)
        }
    }

    /// Enables this program to be used.
    /// Shader MUST remain in scope for duration of usage.
    pub fn get_attribute(&self, name : &str) -> gl::types::GLint {
        let string = CString::new(name).unwrap();
        unsafe {
            gl::GetAttribLocation(self.program, string.as_ptr())
        }
    }

    /// Creates a new shader.
    /// Returns: Shader if compile succeeded, msg if failed.
    pub fn create_shader(vertex : &[u8], frag : &[u8]) -> Result<GLSLShader, String> {
        unsafe {
            let mut status = gl::FALSE as GLint;

            // Create our shader program
            let program = gl::CreateProgram();

            // Create our vertex shader
            let vert_shader = gl::CreateShader(gl::VERTEX_SHADER);

            let vert_ptr = CString::new(vertex).unwrap();

            gl::ShaderSource(vert_shader, 1, [vert_ptr.as_ptr()].as_ptr(), ptr::null());

            gl::CompileShader(vert_shader);
            gl::GetShaderiv(vert_shader, gl::COMPILE_STATUS, &mut status);

            if status == gl::FALSE as GLint {
                let mut len: GLint = 0;
                gl::GetShaderiv(vert_shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf : Vec<u8> = vec![0; len as usize];
                gl::GetShaderInfoLog(vert_shader, len, ptr::null_mut(),
                                      buf.as_mut_ptr() as *mut gl::types::GLchar);
                return Err(String::from_utf8(buf).ok()
                    .expect("ProgramInfoLog not valid utf8"));
            }

            gl::AttachShader(program, vert_shader);

            // Create our fragment shader
            let frag_shader = gl::CreateShader(gl::FRAGMENT_SHADER);

            let frag_ptr = CString::new(frag).unwrap();

            gl::ShaderSource(frag_shader, 1, [frag_ptr.as_ptr()].as_ptr(), ptr::null());

            gl::CompileShader(frag_shader);
            gl::GetShaderiv(frag_shader, gl::COMPILE_STATUS, &mut status);

            if status == gl::FALSE as GLint {
                let mut len: GLint = 0;
                gl::GetShaderiv(frag_shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf : Vec<u8> = vec![0; len as usize];
                gl::GetShaderInfoLog(frag_shader, len, ptr::null_mut(),
                                     buf.as_mut_ptr() as *mut gl::types::GLchar);
                return Err(String::from_utf8(buf).ok()
                    .expect("ProgramInfoLog not valid utf8"));
            }

            gl::AttachShader(program, frag_shader);

            // Compile and link
            gl::LinkProgram(program);

            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

            if status == gl::FALSE as GLint {
                let mut len: GLint = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf : Vec<u8> = vec![0; len as usize];
                gl::GetProgramInfoLog(program, len, ptr::null_mut(),
                                      buf.as_mut_ptr() as *mut gl::types::GLchar);
                Err(String::from_utf8(buf).ok()
                    .expect("ProgramInfoLog not valid utf8"))
            } else {
                Ok(GLSLShader {
                    program,
                    vertex: vert_shader,
                    fragment: frag_shader
                })
            }
        }
    }
}

impl Drop for GLSLShader {
    fn drop(&mut self) {
        println!("Unloading shader!");
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteShader(self.vertex);
            gl::DeleteShader(self.fragment);
        }
    }
}
