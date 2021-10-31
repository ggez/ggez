#version 150 core

uniform sampler2DMS t_Texture;
in vec2 v_Uv;
in vec4 v_Color;
out vec4 Target0;

layout (std140) uniform Globals {
    mat4 u_MVP;
};

layout (std140) uniform Fragments {
    int u_frags;
};

void main() {
    vec2 d = textureSize(t_Texture);
    ivec2 i = ivec2(d * v_Uv);
    Target0 = texelFetch(t_Texture, i, 0);
    for(int j=1; j<u_frags; ++j) {
        Target0 = Target0 + texelFetch(t_Texture, i, j);
    }
    Target0 = Target0 / float(u_frags);
}
