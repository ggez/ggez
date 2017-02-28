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

vec2 rotate(vec2 v, float rad) {
	float s = sin(rad);
	float c = cos(rad);
	mat2 rot = mat2(c, -s, s, c);
	return rot * v;
}

void main() {
    v_Uv = a_Uv;
    gl_Position = vec4((rotate(a_Pos, u_Rotation) * u_Scale) + u_Dest, 0.0, 1.0) * u_Transform;
}
