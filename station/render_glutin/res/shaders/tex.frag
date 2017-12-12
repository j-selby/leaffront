#version 150 core

uniform sampler2D bind_tex;

in vec2 output_uv;
in vec4 output_color;

out vec4 outColor;

void main() {
    outColor = texture2D(bind_tex, output_uv) * output_color;
}