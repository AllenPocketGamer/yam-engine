#version 450

const mat4 MX_CORRECTION = mat4(
    1.0, 0.0, 0.0, 0.0, // column 0
    0.0, 1.0, 0.0, 0.0, // column 1
    0.0, 0.0, 0.5, 0.0, // column 2
    0.0, 0.0, 0.5, 1.0  // column 3
);

layout(location = 0) in vec2 i_position;
layout(location = 1) in vec2 i_complex; // (re, im)
layout(location = 2) in vec2 i_scale;

layout(location = 3) in vec4 v_pos;

layout(push_constant) uniform Matrices {
    mat4 MX_VIEW;
    mat4 MX_PROJECTION;
};

mat4 to_mx_model() {
    return mat4(
        i_complex.x * i_scale.x, i_complex.y * i_scale.x, 0.0, 0.0,     // column 0
        -i_complex.y * i_scale.y, i_complex.x * i_scale.y, 0.0, 0.0,    // column 1
        0.0, 0.0, 1.0, 0.0,                                             // column 2
        i_position.x, i_position.y, 0.0, 1.0                            // column 3
    );
}

void main() {
    gl_Position = MX_CORRECTION * MX_PROJECTION * MX_VIEW * to_mx_model() * v_pos;
}
