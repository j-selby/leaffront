/// Manages an interface for drawing different kinds of images.

use gl_render::shader::GLSLShader;

#[derive(Ord, PartialOrd, Eq, PartialEq)]
enum DrawState {
    Colored,
    Textured
}

pub struct Drawer {
    state : DrawState,
    colored : GLSLShader,
    textured : GLSLShader
}

impl Drawer {
    fn configure_state(&mut self, target : DrawState) {
        if self.state != target {
            match target {
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
}

