#version 450

const mat4 MX_CORRECTION = mat4(
    1.0, 0.0, 0.0, 0.0, // column 0
    0.0, 1.0, 0.0, 0.0, // column 1
    0.0, 0.0, 0.5, 0.0, // column 2
    0.0, 0.0, 0.5, 1.0  // column 3
);

layout(location = 0) in vec4 a_Pos;

layout(set = 0, binding = 0, std140) uniform ToDo {
    mat4 mx_model;
    mat4 mx_view;
    mat4 mx_projection;
};

layout(push_constant) uniform Translation {
    mat4 MX_MODEL;
    mat4 MX_VIEW;
    mat4 MX_PROJECTION;
};

void main() {
    gl_Position = MX_CORRECTION * MX_PROJECTION * MX_VIEW * MX_MODEL * a_Pos;
}