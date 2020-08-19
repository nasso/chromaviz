#version 450

layout(location = 0) in vec4 a_Pos_Size_Hue;
layout(location = 0) out vec2 v_TexCoord;
layout(location = 1) out float v_Size;
layout(location = 2) out float v_Hue;

layout(set = 0, binding = 0) uniform Locals {
    vec2 u_FrameSize;
};

out gl_PerVertex {
    vec4 gl_Position;
};

const vec2 QUAD_VERTICES[4] = {
    {-0.5, -0.5},
    {+0.5, -0.5},
    {-0.5, +0.5},
    {+0.5, +0.5},
};

void main() {
    v_TexCoord = QUAD_VERTICES[gl_VertexIndex % 4] + 0.5;
    v_Size = a_Pos_Size_Hue.z;
    v_Hue = a_Pos_Size_Hue.w;

    vec2 position = a_Pos_Size_Hue.xy + QUAD_VERTICES[gl_VertexIndex % 4] * a_Pos_Size_Hue.z / u_FrameSize;

    // go from (0..1) to (-1..1) coordinates
    position = position * 2 - 1;

    gl_Position = vec4(position, 0.0, 1.0);
}
