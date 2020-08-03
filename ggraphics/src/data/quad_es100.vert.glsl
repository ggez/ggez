#version 100

// Defined per-mesh
uniform mat4 u_Projection;

// Defined per-mesh
uniform mat4 u_ModelTransform;

attribute vec4 a_pos;
attribute vec4 a_normal;
attribute vec4 a_color;
attribute vec2 a_uv;

varying vec4 v_Color;
varying vec2 v_Uv;
void main() {
  mat4 foo = u_ModelTransform * u_Projection;
  gl_Position = vec4((a_pos - 0.5) / 5.0);
  v_Color = vec4((a_pos + a_normal + a_color).xyz, foo[0][0]);
  v_Uv = a_uv;
}

