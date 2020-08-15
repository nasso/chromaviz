#version 450

layout(location = 0) in vec2 a_Pos;

out gl_PerVertex {
    vec4 gl_Position;
};

const vec2 c_Quad[4] = {
    {-0.01, -0.01},
    {+0.01, -0.01},
    {-0.01, +0.01},
    {+0.01, +0.01},
};

void main() {
    vec2 position = a_Pos + c_Quad[gl_VertexIndex % 4];

    gl_Position = vec4(position, 0.0, 1.0);
}
