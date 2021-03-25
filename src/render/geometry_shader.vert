#version 450

// NOTE: STRUCTS AREA

// std430 layout        // offset   align   size
//
// align: 8, size: 24
struct Transform2D {
    vec2 position;      // 0        8       8
    vec2 complex;       // 8        8       8
    vec2 scale;         // 16       8       8
};

// std430 layout        // offset   align   size
//
// align: 16, size: 32
struct Geometry {
    uint types;         // 0        4       4
    uint bcolor;        // 4        4       4
    uint icolor;        // 8        4       4
    float thickness;    // 12       4       4
    vec4 extras;        // 16       4       16
};

// NOTE: BUFFERS AREA

layout(push_constant) uniform CONSTANTS {
    // Transform point from `world space` to `eye space`.
    mat4 MX_VIEW;
    // Transform point from `eye space` to `NDC`.
    mat4 MX_PROJECTION;
    // Transform point from `NDC` to `screen space`.
    mat4 MX_VIEWPORT;
};

readonly layout(std430, binding = 0) buffer Transform2DArray {
    Transform2D t_arr[];
};

readonly layout(std430, binding = 1) buffer GeometryArray {
    Geometry g_arr[];
};

// NOTE: IN VARIABLES

// vertex
layout(location = 0) in vec4 v_pos;
// index.0: transform index; index.1: geometry index.
layout(location = 1) in uvec2 index;

// NOTE: OUT VARIABLES

// The border thickness in world space.
layout(location = 0) out float thickness;
// GeometryType + BorderType + InnerType.
layout(location = 1) out uvec3 types;
// Geometry border color.
layout(location = 2) out vec4 bcolor;
// Geometry inner color.
layout(location = 3) out vec4 icolor;
// Matrix that transforms point from local space to world space.
layout(location = 4) out mat4 mx_l2w;

// NOTE: FUNCTIONS AREA

mat4 to_matrix(vec2 position, vec2 complex, vec2 scale) {
    return mat4(
        complex.x * scale.x, complex.y * scale.x, 0.0, 0.0,     // column 0
        -complex.y * scale.y, complex.x * scale.y, 0.0, 0.0,    // column 1
        0.0, 0.0, 1.0, 0.0,                                     // column 2
        position.x, position.y, 0.0, 1.0                        // column 3
    );
}

mat4 to_matrix(Transform2D t) {
    return to_matrix(t.position, t.complex, t.scale);
}

mat4 to_matrix(vec2 centra, float slength, float angle) {
    float rad = radians(angle);
    vec2 complex = vec2(cos(rad), sin(rad));
    vec2 scale = vec2(slength, slength);

    return to_matrix(centra, complex, scale);
}

vec4 hex_to_color(uint hex) {
    uint r = hex >> 24;
    uint g = hex >> 16 & 0xFF;
    uint b = hex >> 8 & 0xFF;
    uint a = hex & 0xFF;

    return vec4(r, g, b, a) / 255.0;
}

uvec4 unzip_types(uint types) {
    uint gtype = types >> 24;
    uint btype = types >> 16 & 0xFF;
    uint itype = types >> 8 & 0xFF;
    uint order = types & 0xFF;

    return uvec4(gtype, btype, itype, order);
}

void main() {
    uint t_index = index.x;
    uint g_index = index.y;

    Transform2D t = t_arr[t_index];
    Geometry g = g_arr[g_index];

    uvec4 types_with_order = unzip_types(g.types);
    // Place order to v_pos.z as depth.
    uint order = types_with_order.w;

    // Transform point from `local space` to `world space`.
    mat4 mx_to_world = to_matrix(t) * to_matrix(g.extras.xy, g.extras.z, g.extras.w);
    // Transform point from `local space` to `clip space/NDC`.
    //
    // Because the camera is orthographic, so the `clip space` is the same as `NDC`.
    mat4 mx_to_clip = MX_PROJECTION * MX_VIEW * mx_to_world;
    // Transform point from `local space` to `screen space`.
    mat4 mx_to_scrn = MX_VIEWPORT * mx_to_clip;

    types = types_with_order.xyz;
    mx_l2w = mx_to_world;
    thickness = g.thickness;
    bcolor = hex_to_color(g.bcolor);
    icolor = hex_to_color(g.icolor);

    gl_Position = mx_to_clip * vec4(v_pos.xy, float(order) - 255.0, v_pos.w);
}