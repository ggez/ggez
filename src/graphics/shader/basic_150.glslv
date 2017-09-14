#version 150 core

in vec2 a_Pos;
in vec2 a_Uv;

in vec4 a_Src;
in vec2 a_Dest;
in vec2 a_Scale;
in vec2 a_Offset;
in vec2 a_Shear;
in float a_Rotation;

layout (std140) uniform Globals {
    mat4 u_MVP;
    vec4 u_Color;
};

out vec2 v_Uv;

void main() {
    v_Uv = a_Uv * a_Src.zw + a_Src.xy;
    mat2 rotation = mat2(cos(a_Rotation), -sin(a_Rotation), sin(a_Rotation), cos(a_Rotation));
    mat2 shear = mat2(1, a_Shear.x, a_Shear.y, 1);
    vec2 position = (((a_Pos * a_Scale) * shear) + a_Offset) * rotation + a_Dest;

    gl_Position = u_MVP * vec4(position, 0.0, 1.0);
}
