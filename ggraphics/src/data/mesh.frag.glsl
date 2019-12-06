#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec4 in_pos;
layout(location = 1) in vec4 frag_color;
layout(location = 2) in vec4 uv;

layout(location = 0) out vec4 color;

layout(set = 0, binding = 1) uniform texture2D colortex;
layout(set = 0, binding = 2) uniform sampler colorsampler;
layout(push_constant) uniform PushConstantTest {
    mat4 proj;
    mat4 view;
};

void main() {
    color = texture(sampler2D(colortex, colorsampler), uv.xy) * frag_color;
    //color = vec4(1,1,1,1);
}
