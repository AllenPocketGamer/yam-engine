#version 450

// NOTE: CONSTANTS AREA

// GeometryType
const uint GT_CIRCLE = 0;
const uint GT_LINE = 1;
const uint GT_ETRIANGLE = 2;
const uint GT_SQUARE = 3;

// BorderType
const uint BT_NONE = 0;
const uint BT_SOLID = 1;
const uint BT_DASH = 2;
const uint BT_DYN_DASH = 3;
const uint BT_NAVI = 4;
const uint BT_DYN_NAVI = 5;
const uint BT_WARN = 6;
const uint BT_DYN_WARN = 7;

// InnerType
const uint IT_NONE = 0;
const uint IT_SOLID = 1;
const uint IT_DITHER = 2;
const uint IT_DYN_DITHER = 3;

// NOTE: STRUCTS AREA

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

layout(std430, binding = 1) buffer GeometryArray {
    Geometry g_arr[];
};

// NOTE: IN VARIABLES

flat layout(location = 0) in uvec3 types;
flat layout(location = 1) in vec2 centra;
flat layout(location = 2) in vec2 extras;
flat layout(location = 3) in vec4 bcolor;
flat layout(location = 4) in vec4 icolor;

// NOTE: OUT VARIABLES

layout(location = 0) out vec4 o_Target;

// NOTE: FUNCTIONS AREA

bool sample_circle(vec2 pos, vec2 centra, float radius) {
    return radius >= distance(pos, centra);
}

void main() {
    uint gtype = types.x;
    
    if(gtype == GT_CIRCLE) {
        if(sample_circle(gl_FragCoord.xy, centra, extras.x)) {
            o_Target = icolor;
        } else {
            discard;
        }
    } else {
        o_Target = icolor;
    }
}