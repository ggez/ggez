#version 150 core

in vec2 a_Pos;
in vec2 a_Uv;

uniform Transform {
    mat4 u_Transform;
};

uniform RectProperties {
    vec2 u_Offset;
    vec2 u_Size;
    vec2 u_ColorMod;

    vec4 u_Src;
    vec4 u_Dest;
    vec2 u_Center;
    float u_Angle;
    float u_FlipHorizontal;
    float u_FlipVertical;
};

out vec2 v_Uv;

void main() {
    v_Uv = a_Uv;
    gl_Position = vec4((a_Pos * u_Dest.zw) + u_Dest.xy, 0.0, 1.0) * u_Transform;
}
