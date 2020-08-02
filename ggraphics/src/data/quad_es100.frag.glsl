#version 100
precision mediump float;

uniform sampler2D u_Tex;
varying vec4 v_Color;
varying vec2 v_Uv;

void main() {
    gl_FragColor = v_Color * texture(u_Tex, v_Uv);
}

/*
#version 300 es
precision mediump float;
in vec3 vert;
in vec2 tex_coord;
in vec4 frag_color;
uniform sampler2D tex;

layout(location=0) out vec4 color;

void main() {
    // Useful for looking at UV values
    //color = vec4(tex_coord, 0.5, 1.0);

    color = texture(tex, tex_coord) * frag_color;
}
*/
