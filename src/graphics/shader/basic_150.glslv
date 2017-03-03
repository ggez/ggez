#version 150 core

in vec2 a_Pos;
in vec2 a_Uv;

uniform Globals {
    mat4 u_Transform;
    vec4 u_Color;
};

uniform RectProperties {
    vec4 u_Src;
    vec2 u_Dest;
    vec2 u_Scale;
    vec2 u_Offset;
    vec2 u_Shear;
    float u_Rotation;
};

out vec2 v_Uv;

void main() {
    v_Uv = a_Uv * u_Src.zw + u_Src.xy;
    mat2 rotation = mat2(cos(u_Rotation), -sin(u_Rotation), sin(u_Rotation), cos(u_Rotation));
    mat2 shear = mat2(1, u_Shear.x, u_Shear.y, 1);
    vec2 position = (((a_Pos * u_Scale) * shear) + u_Offset) * rotation + u_Dest;
    gl_Position = vec4(position, 0.0, 1.0) * u_Transform;
}
