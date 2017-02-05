#version 150 core

in vec2 a_Pos;

void main() {
    gl_Position = vec4(a_Pos, 0.0, 1.0);
}
