#version 300 es
const vec2 verts[6] = vec2[6](
    vec2(0.0f, 0.0f),
    vec2(1.0f, 1.0f),
    vec2(0.0f, 1.0f),

    vec2(0.0f, 0.0f),
    vec2(1.0f, 0.0f),
    vec2(1.0f, 1.0f)
);
const vec2 uvs[6] = vec2[6](
    vec2(0.0f, 1.0f),
    vec2(1.0f, 0.0f),
    vec2(0.0f, 0.0f),

    vec2(0.0f, 1.0f),
    vec2(1.0f, 1.0f),
    vec2(1.0f, 0.0f)
);

// TODO: We don't actually need layouts here, hmmm.
// Not sure how we want to define these.

// Gotta actually use this dummy value or else it'll get
// optimized out and we'll fail to look it up later.
layout(location = 0) in vec2 vertex_dummy;
layout(location = 1) in vec4 model_color;
layout(location = 2) in vec4 model_src_rect;
layout(location = 3) in vec4 model_dst_rect;
layout(location = 4) in vec2 model_offset;
layout(location = 5) in float model_rotation;
uniform mat4 projection;

out vec2 vert;
out vec2 tex_coord;
out vec4 frag_color;

void main() {
    mat2 rotation = mat2(
            cos(model_rotation), -sin(model_rotation),
            sin(model_rotation), cos(model_rotation)
    );
    vec2 offset_inverse = -model_offset;
    vec2 scale = model_dst_rect.zw;
    vec2 dest_point = model_dst_rect.xy;
    vert = (verts[gl_VertexID % 6] + offset_inverse) * scale * rotation
        + model_offset + dest_point + vertex_dummy;
    /*
    vert = verts[gl_VertexID % 6]
        * model_dst_rect.zw + model_dst_rect.xy
        * rotation + vertex_dummy + model_offset;
        */
    // TODO: Double-check these UV's are correct
    tex_coord = uvs[gl_VertexID] * model_src_rect.zw + model_src_rect.xy;
    frag_color = model_color;
    gl_Position = vec4(vert, 0.0, 1.0) * projection;
}
