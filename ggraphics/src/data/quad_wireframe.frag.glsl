#version 300 es
precision mediump float;
in vec2 vert;
in vec2 tex_coord;
in vec4 frag_color;
in vec4 barys[3];
uniform sampler2D tex;

layout(location=0) out vec4 color;

void main() {
    // Useful for looking at UV values
    //color = vec4(tex_coord, 0.5, 1.0);

    color = texture(tex, tex_coord) * frag_color;
}
