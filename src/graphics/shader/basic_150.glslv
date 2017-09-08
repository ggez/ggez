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
    mat4 u_Projection;
    vec4 u_Color;
};

layout (std140) uniform GlobalTransform {
    vec2 u_Translation;
    vec2 u_Scale;
    vec2 u_Offset;
    vec2 u_Shear;
    float u_Rotation;
};

out vec2 v_Uv;

void main() {
    v_Uv = a_Uv * a_Src.zw + a_Src.xy;
    mat2 instance_rotation = mat2(cos(a_Rotation), -sin(a_Rotation), sin(a_Rotation), cos(a_Rotation));
    mat2 instance_shear = mat2(1, a_Shear.x, a_Shear.y, 1);
    vec2 instance_position = (((a_Pos * a_Scale) * instance_shear) + a_Offset) * instance_rotation + a_Dest;

    mat2 global_rotation = mat2(cos(u_Rotation), -sin(u_Rotation), sin(u_Rotation), cos(u_Rotation));
    mat2 global_shear = mat2(1, u_Shear.x, u_Shear.y, 1);
    vec2 position = (((instance_position - u_Offset) * u_Scale) * global_shear) * global_rotation + u_Offset + u_Translation;

    gl_Position = vec4(position, 0.0, 1.0) * u_Projection;
}
