#version 450

layout(location = 0) out vec2 v_TexCoord;

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
    vec2 pos = QUAD_VERTICES[gl_VertexIndex % 4];
    vec2 uv = pos * 0.5 + 0.5;
    v_TexCoord = vec2(uv.x, 1.0 - uv.y);

    gl_Position = vec4(pos, 0.0, 1.0);
}
