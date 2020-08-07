#version 100
precision mediump float;

// Defined per-mesh
uniform mat4 u_Projection;

// Defined per-mesh
uniform mat4 u_ModelTransform;

attribute vec4 a_pos;
attribute vec4 a_color;
attribute vec2 a_uv;

varying vec4 v_Color;
varying vec2 v_Uv;
void main() {
  mat4 mvp = u_ModelTransform * u_Projection;

  gl_Position = mvp * a_pos;
  v_Color = a_color;
  v_Uv = a_uv;
}

