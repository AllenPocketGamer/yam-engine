#version 450

layout(location = 0) out vec4 o_Target;

void main() {
    o_Target = gl_FragCoord;
}