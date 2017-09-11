uniform sampler2D bind_tex;

varying vec2 output_uv;

void main() {
    gl_FragColor = texture2D(bind_tex, output_uv);
}
