#version 450

layout(location = 0) in vec2 v_TexCoord;
layout(location = 1) in float v_Size;
layout(location = 0) out vec4 outColor;

float circle(vec2 center, float radius) {
    float dist_px = distance(center, v_TexCoord) * v_Size;
    vec2 uv_px = v_TexCoord * v_Size;
    float rad_px = radius * v_Size;

    return 1.0 - smoothstep(rad_px - 1.0, rad_px + 1.0, dist_px);
}

void main() {
    float mask = circle(vec2(0.5, 0.5), 0.25);
    vec3 color = vec3(1.0);

    outColor = vec4(color, mask);
}
