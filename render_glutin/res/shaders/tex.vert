#version 150 core

uniform sampler2D bind_tex;

attribute vec4 input_color;
attribute vec2 input_uv;
attribute vec2 input_vertex;

out vec2 output_uv;
out vec4 output_color;

void main() {
    output_color = input_color;
    output_uv = input_uv;
    gl_Position = vec4(input_vertex, 0.0, 1.0);
}
