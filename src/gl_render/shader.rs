/// Holds and parses GLSL shaders.

use opengles::glesv2 as gl;

pub struct GLSLShader {
    program : gl::GLuint,
    vertex : gl::GLuint,
    fragment : gl::GLuint,
}

impl GLSLShader {
    /// Enables this program to be used.
    /// Shader MUST remain in scope for duration of usage.
    pub fn use_program(&self) {
        gl::use_program(self.program)
    }

    /// Enables this program to be used.
    /// Shader MUST remain in scope for duration of usage.
    pub fn get_attribute(&self, name : &str) -> gl::GLint {
        gl::get_attrib_location(self.program, name)
    }

    /// Creates a new shader.
    /// Returns: Shader if compile succeeded, msg if failed.
    pub fn create_shader(vertex : &[u8], frag : &[u8]) -> Result<GLSLShader, String> {
        // Create our shader program
        let program = gl::create_program();

        // Create our vertex shader
        let vert_shader = gl::create_shader(gl::GL_VERTEX_SHADER);

        gl::shader_source(vert_shader, vertex);

        gl::compile_shader(vert_shader);
        gl::attach_shader(program, vert_shader);

        // Create our fragment shader
        let frag_shader = gl::create_shader(gl::GL_FRAGMENT_SHADER);

        gl::shader_source(frag_shader, frag);

        gl::compile_shader(frag_shader);
        gl::attach_shader(program, frag_shader);

        // Compile and link
        gl::link_program(program);

        match gl::get_program_info_log(program, 8192) {
            Some(msg) => Err(msg),
            None => Ok(GLSLShader {
                program,
                vertex : vert_shader,
                fragment : frag_shader
            })
        }
    }
}

impl Drop for GLSLShader {
    fn drop(&mut self) {
        println!("Unloading shader!");
        gl::delete_program(self.program);
        gl::delete_shader(self.vertex);
        gl::delete_shader(self.fragment);
    }
}
