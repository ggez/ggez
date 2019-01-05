#version 300 es

uniform mediump sampler2D t_Texture;
in mediump vec2 v_Uv;
in mediump vec4 v_Color;
out mediump vec4 Target0;

layout (std140) uniform Globals {
    mediump mat4 u_MVP;
};

void main() {
    Target0 = texture(t_Texture, v_Uv) * v_Color;
}
