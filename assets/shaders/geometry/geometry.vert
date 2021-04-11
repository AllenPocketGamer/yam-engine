#version 450

// NOTE: CONSTANTS AREA

// Geometry2DType
const uint GT_CIRCLE        = 0;
const uint GT_ETRIANGLE     = 1;
const uint GT_SQUARE        = 2;
const uint GT_PENTAGON      = 3;
const uint GT_HEXAGON       = 4;
const uint GT_OCTOGON       = 5;
const uint GT_HEXAGRAM      = 6;
const uint GT_STARFIVE      = 7;
const uint GT_HEART         = 8;

const uint GT_LINE          = 20;
const uint GT_RAY           = 21;
const uint GT_SEGMENT       = 22;

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
    uint datas;         // 0        4       4
    uint bcolor;        // 4        4       4
    uint icolor;        // 8        4       4
    // Positive represents thickness in `screen space`,
    // Negative represents thickness in `local space`.
    float thickness;    // 12       4       4
    vec4 extras;        // 16       4       16
};

// NOTE: BUFFERS AREA

layout(binding = 0) uniform Common {
    // Transform point from `world space` to `eye space`.
    mat4 MX_VIEW;
    // Transform point from `eye space` to `NDC`.
    mat4 MX_PROJECTION;
    // Transform point from `NDC` to `screen space`.
    mat4 MX_VIEWPORT;

    // Delta time
    float t_delta;
    // Total time
    float t_total;
};

readonly layout(std430, binding = 1) buffer Transform2DArray {
    Transform2D t_arr[];
};

readonly layout(std430, binding = 2) buffer GeometryArray {
    Geometry g_arr[];
};

// NOTE: IN VARIABLES

// vertex
layout(location = 0) in vec4 v_pos;
// index.0: transform index; index.1: geometry index.
layout(location = 1) in uvec2 index;

// NOTE: OUT VARIABLES

// The border thickness in `geometry space`.
layout(location = 0) out float th_g;
// GeometryType + BorderDecoration + InnerDecoration.
layout(location = 1) out uvec3 datas;
// Geometry border color.
layout(location = 2) out vec4 bcolor;
// Geometry inner color.
layout(location = 3) out vec4 icolor;
// Matrix that transforms point from `geometry space` to `local space`.
layout(location = 4) out mat4 mx_g2l;
// Matrix that transforms point from `local space` to `world space`.
layout(location = 8) out mat4 mx_l2w;

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

mat4 to_matrix(vec2 centra, float angle, float sl) {
    float rad = radians(angle);
    vec2 complex = vec2(cos(rad), sin(rad));
    vec2 scale = vec2(sl, sl);

    return to_matrix(centra, complex, scale);
}

vec4 hex_to_color(uint hex) {
    uint r = hex >> 24;
    uint g = hex >> 16 & 0xFF;
    uint b = hex >> 8 & 0xFF;
    uint a = hex & 0xFF;

    return vec4(r, g, b, a) / 255.0;
}

uvec4 unzip_datas(uint datas) {
    uint gtype = datas >> 24;
    uint bdeco = datas >> 16 & 0xFF;
    uint ideco = datas >> 8 & 0xFF;
    uint order = datas & 0xFF;

    return uvec4(gtype, bdeco, ideco, order);
}

void main() {
    uint t_index = index.x;
    uint g_index = index.y;

    Transform2D t = t_arr[t_index];
    Geometry g = g_arr[g_index];

    uvec4 datas_with_order = unzip_datas(g.datas);
    // Place order to v_pos.z as depth.
    uint order = datas_with_order.w;

    datas = datas_with_order.xyz;
    bcolor = hex_to_color(g.bcolor);
    icolor = hex_to_color(g.icolor);
    mx_l2w = to_matrix(t);

    const uint gtype = datas.x;

    if(gtype == GT_SEGMENT || gtype == GT_RAY || gtype == GT_LINE) {
        const vec2 ab = g.extras.zw - g.extras.xy;
        const float len = length(ab);

        const vec2 position = (g.extras.xy + g.extras.zw) / 2.0;
        const vec2 complex = ab / len;

        const mat4 mx_l2s = MX_VIEWPORT * MX_PROJECTION * MX_VIEW * mx_l2w;
        const float th_l = g.thickness >= 0 ? g.thickness / length(mx_l2s * vec4(1.0, 0.0, 0.0, 0.0)) : abs(g.thickness);

        const vec2 scale = vec2(len, th_l);

        mx_g2l = to_matrix(position, complex, scale);
        th_g = 1.0;
    } else {
        mx_g2l = to_matrix(g.extras.xy, g.extras.z, g.extras.w);

        const mat4 matrix = g.thickness >= 0 ? MX_VIEWPORT * MX_PROJECTION * MX_VIEW * mx_l2w * mx_g2l : mx_g2l;
        th_g = abs(g.thickness) / length(matrix * vec4(normalize(v_pos.xy), 0.0, 0.0));
    }

    gl_Position = MX_PROJECTION * MX_VIEW * mx_l2w * mx_g2l * vec4(v_pos.xy, float(order) - 255.0, v_pos.w);
}