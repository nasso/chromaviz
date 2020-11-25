#version 450

layout(location = 0) in vec2 v_TexCoord;
layout(location = 0) out vec4 outColor;
layout(set = 1, binding = 5) uniform texture2D t_Color;
layout(set = 1, binding = 6) uniform sampler s_Color;

void main() {
    outColor = texture(sampler2D(t_Color, s_Color), v_TexCoord);
}
