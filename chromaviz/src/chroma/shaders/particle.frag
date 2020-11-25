#version 450

layout(location = 0) in vec2 v_TexCoord;
layout(location = 1) in float v_Size;
layout(location = 2) in float v_Hue;
layout(location = 0) out vec4 outColor;

// All components are in the range [0…1], including hue.
vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

float circle(vec2 center, float radius) {
    float dist_px = distance(center, v_TexCoord) * v_Size;
    vec2 uv_px = v_TexCoord * v_Size;
    float rad_px = radius * v_Size;

    return 1.0 - smoothstep(rad_px - 1.0, rad_px + 1.0, dist_px);
}

void main() {
    float mask = circle(vec2(0.5, 0.5), 0.25);
    vec3 color = hsv2rgb(vec3(v_Hue, 1.0, 1.0));

    outColor = vec4(color, mask);
}
