#version 450

// Geometry Type Enum
const uint CIRCLE = 0;
const uint LINE = 1;
const uint ETRIANGLE = 2;
const uint SQUARE = 3;

const mat4 MX_CORRECTION = mat4(
    1.0, 0.0, 0.0, 0.0, // column 0
    0.0, 1.0, 0.0, 0.0, // column 1
    0.0, 0.0, 0.5, 0.0, // column 2
    0.0, 0.0, 0.5, 1.0  // column 3
);

struct Transform2D {
    vec2 position;
    vec2 complex;
    vec2 scale;
};

mat4 to_mx_model(Transform2D t) {
    return mat4(
        t.complex.x * t.scale.x, t.complex.y * t.scale.x, 0.0, 0.0,     // column 0
        -t.complex.y * t.scale.y, t.complex.x * t.scale.y, 0.0, 0.0,    // column 1
        0.0, 0.0, 1.0, 0.0,                                             // column 2
        t.position.x, t.position.y, 0.0, 1.0                            // column 3
    );
}

// TODO: 对齐有问题, 得仔细调试
struct Geometry {
    uint types;
    uint b_color_hex;
    uint i_color_hex;
    float thickness;
    float[4] extra;
};

// vertex
layout(location = 0) in vec4 v_pos;

// index.0: transform index; index.1: geometry index.
layout(location = 1) in uvec2 index;

layout(push_constant) uniform Matrices {
    mat4 MX_VIEW;
    mat4 MX_PROJECTION;
};

layout(std430, binding = 0) buffer Transform2DArray {
    Transform2D t_arr[];
};

layout(std430, binding = 1) buffer GeometryArray {
    Geometry g_arr[];
};

layout(location = 0) out vec4 v_color;

void main() {
    uint t_index = index.x;
    uint g_index = index.y;

    uint gtype = g_arr[g_index].types >> 24;

    if(gtype == CIRCLE) {
        // red
        v_color = vec4(1.0, 0.0, 0.0, 1.0);
    } else if (gtype == ETRIANGLE) {
        // green
        v_color = vec4(0.0, 1.0, 0.0, 1.0);
    } else if (gtype == SQUARE) {
        // blue
        v_color = vec4(0.0, 0.0, 1.0, 1.0);
    } else {
        // magenta
        v_color = vec4(1.0, 0.0, 1.0, 1.0);
    }

    gl_Position = MX_CORRECTION * MX_PROJECTION * MX_VIEW * to_mx_model(t_arr[t_index]) * v_pos;
}