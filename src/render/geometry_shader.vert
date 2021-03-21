#version 450

// Geometry Type Enum
const uint CIRCLE = 0;
const uint LINE = 1;
const uint ETRIANGLE = 2;
const uint SQUARE = 3;

vec4 hex_to_color(uint hex) {
    uint r = hex >> 24;
    uint g = hex >> 16 & 0xFF;
    uint b = hex >> 8 & 0xFF;
    uint a = hex & 0xFF;

    return vec4(r, g, b, a) / 255.0;
}

mat4 to_matrix(vec2 position, vec2 complex, vec2 scale) {
    return mat4(
        complex.x * scale.x, complex.y * scale.x, 0.0, 0.0,     // column 0
        -complex.y * scale.y, complex.x * scale.y, 0.0, 0.0,    // column 1
        0.0, 0.0, 1.0, 0.0,                                     // column 2
        position.x, position.y, 0.0, 1.0                        // column 3
    );
}

// std430 layout        // offset   align   size
//
// align: 8, size: 24
struct Transform2D {
    vec2 position;      // 0        8       8
    vec2 complex;       // 8        8       8
    vec2 scale;         // 16       8       8
};

mat4 to_matrix(Transform2D t) {
    return to_matrix(t.position, t.complex, t.scale);
}

// std430 layout        // offset   align   size
//
// align: 16, size: 32
struct Geometry {
    uint types;         // 0        4       4
    uint bcolor;        // 4        4       4
    uint icolor;        // 8        4       4
    float thickness;    // 12       4       4
    vec4 extra;         // 16       4       16
};

uvec4 unzip_types(uint types) {
    uint gtype = types >> 24;
    uint btype = types >> 16 & 0xFF;
    uint itype = types >> 8 & 0xFF;
    uint order = types & 0xFF;

    return uvec4(gtype, btype, itype, order);
}

mat4 to_matrix(vec2 centra, float radius, float angle) {
    vec2 complex = vec2(cos(angle), sin(angle));
    vec2 scale = radius * vec2(2.0, 2.0);

    return to_matrix(centra, complex, scale);
}

// vertex
layout(location = 0) in vec4 v_pos;
// index.0: transform index; index.1: geometry index.
layout(location = 1) in uvec2 index;

layout(push_constant) uniform CONSTANTS {
    vec4 VIEWPORT;  // (x, y, w, h)
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
layout(location = 1) out vec4 v_tmp;        // xy: centra, z: radius, thickness

void main() {
    uint t_index = index.x;
    uint g_index = index.y;

    Transform2D t = t_arr[t_index];
    Geometry g = g_arr[g_index];

    uint gtype = unzip_types(g.types).x;

    mat4 mx_model = to_matrix(t) * to_matrix(g.extra.xy, g.extra.z, g.extra.w);

    vec2 centra_wp = (mx_model * vec4(0, 0, 0, 1)).xy;
    float radius_wp = length((mx_model * vec4(0.5, 0, 0, 0)));
    
    v_color = hex_to_color(g.icolor);

    v_tmp = vec4(centra_wp, radius_wp, gtype);

    gl_Position = MX_PROJECTION * MX_VIEW * mx_model * v_pos;
}