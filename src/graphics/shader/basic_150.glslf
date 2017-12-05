#version 150 core

uniform sampler2D t_Texture;
in vec2 v_Uv;
out vec4 Target0;

layout (std140) uniform Globals {
    mat4 u_MVP;
    vec4 u_Color;
};

void main() {
    //Target0 = vec4(1.0, 1.0, 1.0, 1.0);
    Target0 = texture(t_Texture, v_Uv) * u_Color;
}
