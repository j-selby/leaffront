attribute vec4 input_color;
attribute vec2 input_vertex;

varying vec4 output_color;

void main() {
    output_color = input_color;
    gl_Position = vec4(input_vertex, 0.0, 1.0);
}
