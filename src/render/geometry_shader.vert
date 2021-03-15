#version 450

const mat4 MX_CORRECTION = mat4(
    1.0, 0.0, 0.0, 0.0, // column 0
    0.0, 1.0, 0.0, 0.0, // column 1
    0.0, 0.0, 0.5, 0.0, // column 2
    0.0, 0.0, 0.5, 1.0  // column 3
);

// vertex
layout(location = 0) in vec4 v_pos;

// transform2d instances
layout(location = 1) in vec2 i_position;
layout(location = 2) in vec2 i_complex;
layout(location = 3) in vec2 i_scale;

// geometry instances
layout(location = 4) in uvec4 types;        // geometry type + border type + inner type + order
layout(location = 5) in vec4 border_color;
layout(location = 6) in vec4 inner_color;
layout(location = 7) in float thickness;    // border thinckess
layout(location = 8) in vec4 extra;         // `centra + radius + angle` or `point_a + point_b` 

layout(push_constant) uniform Matrices {
    mat4 MX_VIEW;
    mat4 MX_PROJECTION;
};

// convert `transform2d instance` to transformation(model) matrix.
mat4 to_mx_model() {
    return mat4(
        i_complex.x * i_scale.x, i_complex.y * i_scale.x, 0.0, 0.0,     // column 0
        -i_complex.y * i_scale.y, i_complex.x * i_scale.y, 0.0, 0.0,    // column 1
        0.0, 0.0, 1.0, 0.0,                                 // column 2
        i_position.x, i_position.y, 0.0, 1.0                // column 3
    );
}

void main() {
    gl_Position = MX_CORRECTION * MX_PROJECTION * MX_VIEW * to_mx_model() * v_pos;
}