#version 450

// NOTE: CONSTANTS AREA

// Geometry Type Enum
const uint CIRCLE = 0;
const uint LINE = 1;
const uint ETRIANGLE = 2;
const uint SQUARE = 3;

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
    // Transform point from `World` space to `Eye` space.
    mat4 MX_VIEW;
    // Transform point from `Eye` space to `NDC`.
    mat4 MX_PROJECTION;
    // Transform point from `NDC` to `Screen` space.
    mat4 MX_VIEWPORT;
};

// vertex
layout(location = 0) in vec4 v_pos;
// index.0: transform index; index.1: geometry index.
layout(location = 1) in uvec2 index;

readonly layout(std430, binding = 0) buffer Transform2DArray {
    Transform2D t_arr[];
};

readonly layout(std430, binding = 1) buffer GeometryArray {
    Geometry g_arr[];
};

// NOTE: OUT VARIABLES

// GeometryType + BorderType + InnerType.
layout(location = 0) out uvec3 types;
// Why not calculate `centra` and `extras` in fragment shader?
//
// Of course you can calculate `centra` and `extras` in fragment shader.
//
// look likes:
//  centra_ss = MX_PROJECTION * MX_VIEW * mx_transform2d * mx_geometry * vec4(0, 0, 0, 1); 
//  ...
//
// BUT, it will damage the performance(Especially when it has to render tons of geometry)!
//
// Because of `centra` and `extras` are uniform in warp, so i place the calculation code
// in vertex shader. 
//
// (IN SCREEN SPACE) the centra point of the quad.
layout(location = 1) out vec2 centra;
// (IN SCREEN SPACE) radius + thickness / radius + another
layout(location = 2) out vec2 extras;
// Geometry border color.
layout(location = 3) out vec4 bcolor;
// Geometry inner color.
layout(location = 4) out vec4 icolor;

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

mat4 to_matrix(vec2 centra, float radius, float angle) {
    vec2 complex = vec2(cos(angle), sin(angle));
    vec2 scale = radius * vec2(2.0, 2.0);

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

    // Transform point from `local space` to `clip space/NDC`.
    //
    // Because the camera is orthographic, so the `clip space` is the same as `NDC`.
    mat4 mx_to_clip = MX_PROJECTION * MX_VIEW * to_matrix(t) * to_matrix(g.extras.xy, g.extras.z, g.extras.w);
    // Transform point from `local space` to `screen space`.
    mat4 mx_to_scrn = MX_VIEWPORT * mx_to_clip;

    types = types_with_order.xyz;
    centra = (mx_to_scrn * vec4(0.0, 0.0, 0.0, 1.0)).xy;
    
    float radius = length(mx_to_scrn * vec4(0.5, 0.0, 0.0, 0.0));
    float thickness = length(mx_to_scrn * vec4(g.thickness, 0.0, 0.0, 0.0));
    extras = vec2(radius, thickness);
    
    bcolor = hex_to_color(g.bcolor);
    icolor = hex_to_color(g.icolor);

    gl_Position = mx_to_clip * vec4(v_pos.xy, order, v_pos.w);
}