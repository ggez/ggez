#version 300 es

// This shader should compile, but it shouldn't really work anywhere
// as sampler2DMS isn't used in GLSL ES 300 yet.

uniform mediump sampler2DMS t_Texture;
in mediump vec2 v_Uv;
in mediump vec4 v_Color;
out mediump vec4 Target0;

layout (std140) uniform Globals {
    mediump mat4 u_MVP;
};

layout (std140) uniform Fragments {
    int u_frags;
};

void main() {
    ivec2 d = textureSize(t_Texture);
    ivec2 i = d * ivec2(v_Uv);
    Target0 = texelFetch(t_Texture, i, 0);
    for(int j=1; j<u_frags; ++j) {
        Target0 = Target0 + texelFetch(t_Texture, i, j);
    }
    Target0 = Target0 / float(u_frags);
}
