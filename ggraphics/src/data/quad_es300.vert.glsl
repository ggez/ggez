#version 300 es
const vec3 verts[6] = vec3[6](
    vec3(0.0f, 0.0f, 0.0),
    vec3(1.0f, 1.0f, 0.0),
    vec3(0.0f, 1.0f, 0.0),

    vec3(0.0f, 0.0f, 0.0),
    vec3(1.0f, 0.0f, 0.0),
    vec3(1.0f, 1.0f, 0.0)
);
const vec2 uvs[6] = vec2[6](
    vec2(0.0f, 0.0f),
    vec2(1.0f, 1.0f),
    vec2(0.0f, 1.0f),

    vec2(0.0f, 0.0f),
    vec2(1.0f, 0.0f),
    vec2(1.0f, 1.0f)
);

// TODO: We don't actually need layouts here, hmmm.
// Not sure how we want to define these.

// Gotta actually use this dummy value or else it'll get
// optimized out and we'll fail to look it up later.
layout(location = 0) in vec3 vertex_dummy;
layout(location = 1) in vec4 model_color;
layout(location = 2) in vec4 model_src_rect;
layout(location = 3) in vec4 model_dst_rect;
layout(location = 4) in vec2 model_offset;
layout(location = 5) in float model_rotation;
uniform mat4 projection;

out vec3 vert;
out vec2 tex_coord;
out vec4 frag_color;

void main() {
    mat3 rotation = mat3(
            cos(model_rotation), -sin(model_rotation), 0.0,
            sin(model_rotation), cos(model_rotation), 0.0,
            0.0, 0.0, 1.0
    );
    vec3 offset_inverse = -vec3(model_offset, 0.0);
    vec3 scale = vec3(model_dst_rect.zw, 1.0);
    vec3 dest_point = vec3(model_dst_rect.xy, 0.0);
    vert = (verts[gl_VertexID % 6] + offset_inverse) * scale * rotation
        + vec3(model_offset, 0.0) + dest_point + vertex_dummy;

    // TODO: Double-check these UV's are correct
    tex_coord = uvs[gl_VertexID] * model_src_rect.zw + model_src_rect.xy;
    frag_color = model_color;
    gl_Position = vec4(vert, 1.0) * projection;
}
