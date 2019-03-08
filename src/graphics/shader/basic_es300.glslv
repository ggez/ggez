#version 300 es

in mediump vec2 a_Pos;
in mediump vec2 a_Uv;
in mediump vec4 a_VertColor;

in mediump vec4 a_Src;
in mediump vec4 a_TCol1;
in mediump vec4 a_TCol2;
in mediump vec4 a_TCol3;
in mediump vec4 a_TCol4;
in mediump vec4 a_Color;

layout (std140) uniform Globals {
    mediump mat4 u_MVP;
};

out mediump vec2 v_Uv;
out mediump vec4 v_Color;

void main() {
    v_Uv = a_Uv * a_Src.zw + a_Src.xy;
    v_Color = a_Color * a_VertColor;
    mat4 instance_transform = mat4(a_TCol1, a_TCol2, a_TCol3, a_TCol4);
    vec4 position = instance_transform * vec4(a_Pos, 0.0, 1.0);

    gl_Position = u_MVP * position;
}
