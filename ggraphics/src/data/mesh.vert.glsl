#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 pos;
layout(location = 1) in vec4 color;
layout(location = 2) in vec3 norm;
// per-instance data below here?.
// vec4[4] is used instead of mat4 due to spirv-cross bug for dx12 backend
layout(location = 3) in vec4 model[4];
// Skip some locations for the members of the vec above
layout(location = 7) in vec4 src;
layout(location = 8) in vec4 model_color;

layout(push_constant) uniform PushConstantTest {
    mat4 proj;
    mat4 view;
};


layout(location = 0) out vec4 frag_pos;
layout(location = 1) out vec4 frag_color;
layout(location = 2) out vec4 uv;

void main() {
    mat4 model_mat = mat4(model[0], model[1], model[2], model[3]);

    frag_color = color;
    frag_pos = model_mat * vec4(pos, 1.0);
    // TODO: Have unit quad's and scale verts properly by some multiplier,
    // instead of having to divide by the size of the quad
    // We also invert Y here to make things right-side up.
    uv = vec4(pos.x / 100.0, 1 - (pos.y / 100.0), 0, 0);

    // TODO: Fix depth crap!
    gl_Position = proj * view * frag_pos;
    gl_Position.z = 0.5;
}
