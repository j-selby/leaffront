uniform sampler2D bind_tex;

varying vec2 output_uv;
varying vec4 output_color;

void main() {
    gl_FragColor = texture2D(bind_tex, output_uv) * output_color;
}