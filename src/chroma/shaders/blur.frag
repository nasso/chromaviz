#version 450

layout(set = 0, binding = 0) uniform Locals {
    vec2 u_FrameSize;
};

layout(location = 0) in vec2 v_TexCoord;
layout(location = 0) out vec4 outColor;
layout(set = 1, binding = 5) uniform texture2D t_Color;
layout(set = 1, binding = 6) uniform sampler s_Color;

// adapted from Jam3/glsl-fast-gaussian-blur (MIT)
vec4 blur9(texture2D tex, sampler smp, vec2 resolution, vec2 direction) {
    vec4 color = vec4(0.0);
    vec2 off1 = vec2(1.3846153846) * direction;
    vec2 off2 = vec2(3.2307692308) * direction;

    color += texture(sampler2D(tex, smp), v_TexCoord) * 0.2270270270;
    color += texture(sampler2D(tex, smp), v_TexCoord + (off1 / resolution)) * 0.3162162162;
    color += texture(sampler2D(tex, smp), v_TexCoord - (off1 / resolution)) * 0.3162162162;
    color += texture(sampler2D(tex, smp), v_TexCoord + (off2 / resolution)) * 0.0702702703;
    color += texture(sampler2D(tex, smp), v_TexCoord - (off2 / resolution)) * 0.0702702703;

    return color;
}

void vertical() {
    outColor = blur9(t_Color, s_Color, u_FrameSize, vec2(0.0, 1.0));
}

void horizontal() {
    outColor = blur9(t_Color, s_Color, u_FrameSize, vec2(1.0, 0.0));
}

void main() {
#ifdef BLUR_VERTICAL
    vertical();
#else
    horizontal();
#endif
}
