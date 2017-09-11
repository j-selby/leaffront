extern crate egl;
extern crate opengles;
extern crate videocore;

extern crate image;
extern crate rusttype;

extern crate chrono;

extern crate fps_counter;

mod color;
mod texture;

mod gl_render;
mod pi;

use opengles::glesv2 as gl;

use rusttype::{FontCollection, Scale, point};

use color::Color;
use texture::Texture;

use gl_render::texture::GlTexture;
use gl_render::shader::GLSLShader;

use pi::gl_context::Context;

use image::load_from_memory;

use std::collections::BTreeMap;

use chrono::Local;

struct TextureCommand {
    pub tex_ptr  : GlTexture,
    pub vertices : Vec<f32>
}

fn gl_loop(context: Context) {
    let dimensions = Context::get_resolution();

    gl::viewport(0, 0, dimensions.width as i32, dimensions.height as i32);

    // init shaders
    let program = GLSLShader::create_shader(include_bytes!("../res/shaders/tex.vert"),
                                            include_bytes!("../res/shaders/tex.frag")).unwrap();

    program.use_program();

    // get attributes
    let input_uv = program.get_attribute("input_uv");
    let input_vertex = program.get_attribute("input_vertex");

    // load triangle vertex data into buffer
    let (vertex_vbo, texture_vbo) = init_triangle();

    // load background image
    println!("Load image:");
    let bg_image = load_from_memory(include_bytes!("../res/bg.png")).unwrap();
    println!("Convert");

    println!("Upload");

    println!("font");

    // load rusttype font into memory
    let font_data = include_bytes!("../res/opensans.ttf");

    let collection = FontCollection::from_bytes(font_data as &[u8]);
    let font = collection.into_font().unwrap(); // only succeeds if collection consists of one font


    let mut counter = fps_counter::FPSCounter::new();

    let mut map = BTreeMap::new();

    let bg_cmd = TextureCommand {
        vertices : [
            -1.0,   1.0,
            -1.0,  -1.0,
            1.0,  -1.0,

            -1.0,  -1.0,
            1.0,   1.0,
            1.0,  -1.0
        ].to_vec(),
        tex_ptr : GlTexture::from_image(&bg_image.to_rgba())
    };

    for _ in 0 .. 5 {
        gl::clear_color(0.0, 1.0, 0.0, 1.0);
        gl::clear(gl::GL_COLOR_BUFFER_BIT);

        program.use_program();

        gl::enable_vertex_attrib_array(input_uv as gl::GLuint);
        gl::enable_vertex_attrib_array(input_vertex as gl::GLuint);

        //gl::disable(gl::GL_CULL_FACE);
        gl::enable(gl::GL_BLEND);
        gl::blend_func(gl::GL_SRC_ALPHA, gl::GL_ONE_MINUS_SRC_ALPHA);

        gl::bind_buffer(gl::GL_ARRAY_BUFFER, texture_vbo);
        gl::vertex_attrib_pointer_offset(input_uv as gl::GLuint, 2,
                                         gl::GL_FLOAT, false, 0, 0);

        gl::bind_buffer(gl::GL_ARRAY_BUFFER, vertex_vbo);
        gl::vertex_attrib_pointer_offset(input_vertex as gl::GLuint, 2,
                                         gl::GL_FLOAT, false, 0, 0);


        //gl::bind_buffer(gl::GL_ARRAY_BUFFER, vertex_vbo);
        gl::buffer_data(gl::GL_ARRAY_BUFFER, &bg_cmd.vertices, gl::GL_STATIC_DRAW);

        gl::active_texture(gl::GL_TEXTURE_2D);
        gl::bind_texture(gl::GL_TEXTURE_2D, bg_cmd.tex_ptr.get_raw_ptr());

        gl::draw_arrays(gl::GL_TRIANGLE_FAN, 0, 6);

        {
            let time = Local::now();
            let time = time.format("%I:%M:%S %P").to_string();
            let msg = format!("FPS: {}\n{}", counter.tick(), time);
            let layout = font.layout(&msg,
                                     Scale { x: 60.0, y: 50.0 },
                                     point(20.0, 50.0));

            let base_color = Color::new_3byte(255, 255, 255);
            for letter in layout {
                // Render out texture
                let bounding_box_opt = letter.pixel_bounding_box();

                if bounding_box_opt.is_none() {
                    continue;
                }

                let bounding_box = bounding_box_opt.unwrap();

                // See if we already have this letter
                if !map.contains_key(&letter.id()) {
                    let mut tex = Texture::new(bounding_box.width() as usize,
                                               bounding_box.height() as usize);

                    {
                        let render_pos = |x: u32, y: u32, factor: f32| {
                            tex.draw_pixel(&base_color.alpha((factor * 255.0) as u8),
                                           x as usize, y as usize);
                        };

                        letter.draw(render_pos);
                    }

                    let opengl_tex = GlTexture::from_texture(&tex);

                    map.insert(letter.id(), opengl_tex);
                }

                // Size the texture for a OpenGL environment
                let min_x = (bounding_box.min.x as f32) / dimensions.width as f32 * 2.0 - 1.0;
                let max_x = (bounding_box.max.x as f32) / dimensions.width as f32 * 2.0 - 1.0;
                let min_y = (bounding_box.min.y as f32) / dimensions.height as f32 * 2.0 - 1.0;
                let max_y = (bounding_box.max.y as f32) / dimensions.height as f32 * 2.0 - 1.0;

                // Generate vertex data
                let vertices = [
                    // Vertex 1
                    min_x, -min_y,
                    min_x, -max_y,
                    max_x, -max_y,
                    // Vertex 2
                    min_x, -max_y,
                    max_x, -min_y,
                    max_x, -max_y,
                ];

                //println!("Vertices: {:?}", vertices);
                //println!("Allocated texture of size: {} {}", tex.get_width(), tex.get_height());

                /*let cmd = TextureCommand {
                    tex_ptr: opengl_tex,
                    vertices: vertices.to_vec()
                };*/

                let tex = map.get(&letter.id()).unwrap();


                // Setup texture UV data
                // Setup vertice data
                gl::bind_buffer(gl::GL_ARRAY_BUFFER, vertex_vbo);
                gl::buffer_data(gl::GL_ARRAY_BUFFER, &vertices, gl::GL_STATIC_DRAW);

                gl::active_texture(gl::GL_TEXTURE_2D);
                gl::bind_texture(gl::GL_TEXTURE_2D, tex.get_raw_ptr());

                gl::draw_arrays(gl::GL_TRIANGLE_FAN, 0, 6);
                //commands.push(cmd);
            }
        }

        //println!("Render: {}", counter.tick());

        gl::disable_vertex_attrib_array(input_vertex as gl::GLuint);
        gl::disable_vertex_attrib_array(input_uv as gl::GLuint);

        gl::bind_buffer(gl::GL_ARRAY_BUFFER, 0);

        // swap graphics buffers
        if !context.swap_buffers() {
            println!("Failed to swap buffers!");
        }

        std::thread::sleep(std::time::Duration::new(1, 0));
    }
}


fn init_triangle() -> (gl::GLuint, gl::GLuint) {
    // generate a buffer to hold the vertices and UVs
    let vbos = gl::gen_buffers(2);

    // texture UVs
    let uv : [f32; 12] = [
        0.0, 0.0,
        0.0,  1.0,
        1.0,  1.0,
        0.0,  1.0,
        1.0, 0.0,
        1.0,  1.0,
    ];

    gl::bind_buffer(gl::GL_ARRAY_BUFFER, vbos[1]);
    gl::buffer_data(gl::GL_ARRAY_BUFFER, &uv, gl::GL_STATIC_DRAW);

    (vbos[0], vbos[1])
}

fn main() {
    // init egl

    let context = Context::build().unwrap();

    gl_loop(context);
}
