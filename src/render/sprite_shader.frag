#version 450

layout(location = 0) out vec4 o_Target;

layout(set = 0, binding = 0, std140) uniform Color {
    uint hex;
};

vec4 to_color(uint hex) {
    return uvec4(hex >> 24, (hex >> 16) & 0xFF, (hex >> 8) & 0xFF, hex & 0xFF) / 255.0;
}

void main() {
    o_Target = to_color(hex);
    // o_Target = vec4(0, 0.5, 1.0, 1.0);
}