#version 450

layout(location = 0) out vec4 o_Target;

layout(set = 0, binding = 0, std140) uniform Color {
    vec4 color;
};

void main() {
    o_Target = color;
}