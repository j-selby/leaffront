uniform sampler2D bind_tex;

attribute vec2 input_uv;
attribute vec2 input_vertex;

varying vec2 output_uv;

void main() {
    output_uv = input_uv;
    gl_Position = vec4(input_vertex, 0.0, 1.0);
}
