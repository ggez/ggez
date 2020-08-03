#version 100
precision mediump float;

// Defined per-draw-call
uniform mat4 u_Projection;

// Defined per-mesh
uniform mat4 u_ModelTransform;

uniform sampler2D u_Tex;

varying vec4 v_Color;
varying vec2 v_Uv;

void main() {
  gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0) + v_Color + texture2D(u_Tex, v_Uv);
}
