#version 450

layout(location = 0) in vec4 a_Pos;

out gl_PerVertex {
    vec4 gl_Position;
};

layout(set = 0, binding = 0, std140) uniform Translations {
    mat4 mx_model;
    mat4 mx_view;
    mat4 mx_projection;
};

void main() {
    mat4 mx_correction = mat4(
        1.0, 0.0, 0.0, 0.0, // column 0
        0.0, 1.0, 0.0, 0.0, // column 1
        0.0, 0.0, 0.5, 0.0, // column 2
        0.0, 0.0, 0.5, 1.0  // column 3
    );

    gl_Position = mx_correction * mx_projection * mx_view * mx_model * a_Pos;
}