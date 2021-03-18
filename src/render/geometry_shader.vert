#version 450

const mat4 MX_CORRECTION = mat4(
    1.0, 0.0, 0.0, 0.0, // column 0
    0.0, 1.0, 0.0, 0.0, // column 1
    0.0, 0.0, 0.5, 0.0, // column 2
    0.0, 0.0, 0.5, 1.0  // column 3
);

// vertex
layout(location = 0) in vec4 v_pos;

// index.0: transform index; index.1: geometry index.
layout(location = 1) in uvec2 index;

// geometry instances
// layout(location = 3) in uvec4 types;        // geometry type + border type + inner type + order
// layout(location = 4) in vec4 border_color;
// layout(location = 5) in vec4 inner_color;
// layout(location = 6) in float thickness;    // border thinckess
// layout(location = 7) in vec4 extra;         // `centra + radius + angle` or `point_a + point_b` 

layout(push_constant) uniform Matrices {
    mat4 MX_VIEW;
    mat4 MX_PROJECTION;
};

layout(std430, binding = 0) buffer Test {
    vec2 k_position;
    vec2 k_complex;
    vec2 k_scale;
};

mat4 to_mx_model(vec2 pos, vec2 compl, vec2 scale) {
    return mat4(
        compl.x * scale.x, compl.y * scale.x, 0.0, 0.0,     // column 0
        -compl.y * scale.y, compl.x * scale.y, 0.0, 0.0,    // column 1
        0.0, 0.0, 1.0, 0.0,                                 // column 2
        pos.x, pos.y, 0.0, 1.0                              // column 3
    );
}

void main() {
    gl_Position = MX_CORRECTION * MX_PROJECTION * MX_VIEW * to_mx_model(k_position, k_complex, k_scale) * v_pos;
}