#version 450

layout(location = 0) in vec3 a_Pos_Size;
layout(location = 0) out vec2 v_TexCoord;

layout(set = 0, binding = 0) uniform Globals {
    vec2 u_FrameSize;
};

out gl_PerVertex {
    vec4 gl_Position;
};

const vec2 QUAD_VERTICES[4] = {
    {-1.0, -1.0},
    {+1.0, -1.0},
    {-1.0, +1.0},
    {+1.0, +1.0},
};

void main() {
    v_TexCoord = QUAD_VERTICES[gl_VertexIndex % 4] * 0.5 + 0.5;

    vec2 position = a_Pos_Size.xy + QUAD_VERTICES[gl_VertexIndex % 4] * a_Pos_Size.z / 2 / u_FrameSize;

    // go from (0..1) to (-1..1) coordinates
    position = position * 2 - 1;

    gl_Position = vec4(position, 0.0, 1.0);
}
