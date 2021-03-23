#version 450

// NOTE: CONSTANTS AREA

// GeometryType
const uint GT_CIRCLE    = 0;
const uint GT_LINE      = 1;
const uint GT_ETRIANGLE = 2;
const uint GT_SQUARE    = 3;
const uint GT_PENTAGON  = 4;
const uint GT_HEXAGON   = 5;
const uint GT_OCTOGON   = 6;
const uint GT_HEXAGRAM  = 7;
const uint GT_STARFIVE  = 8;
const uint GT_HEART     = 9;

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

flat layout(location = 0) in float thickness;
flat layout(location = 1) in uvec3 types;
flat layout(location = 2) in vec4 bcolor;
flat layout(location = 3) in vec4 icolor;
flat layout(location = 4) in mat4 mx_s2l;

// NOTE: OUT VARIABLES

layout(location = 0) out vec4 o_Target;

// NOTE: FUNCTIONS AREA

// SDF函数可以参考这里: https://www.iquilezles.org/www/articles/distfunctions2d/distfunctions2d.htm

// sl(side length) ∈ [0, 1].
float sdf_circle(vec2 pos, float sl) {
    float radius = 0.5 * sl;
    
    return radius - length(pos);
}

// sl(side length) ∈ [0, 1].
float sdf_etriangle(vec2 pos, float sl) {
    const float K = sqrt(3.0);

    // The radius of the circle enclosing the etriangle.
    float radius = 0.5 * sl;
    // the side length of etriangle.
    float etsl = K * radius;

    // Map points along the Y axis.
    pos.x = abs(pos.x) - 0.5 * etsl;
    pos.y = pos.y + 0.5 * radius;
    // Map points along the `l: x + √3y = 0` axis.
    pos = pos.x + K * pos.y > 0.0 ? vec2(pos.x - K * pos.y, -K * pos.x - pos.y) / 2.0 : pos;
    
    pos.x -= clamp(pos.x, -etsl, 0.0);

    return length(pos) * sign(pos.y);
}

// sl(side length) ∈ [0, 1].
float sdf_square(vec2 pos, float sl) {
    pos = abs(pos);
    pos = pos.x - pos.y > 0.0 ? pos = pos.yx : pos;
    pos.y -= 0.5 * sl;

    pos.x -= clamp(pos.x, 0, 0.5 * sl);
    
    return length(pos) * sign(-pos.y);
}

// sl(side length) ∈ [0, 1].
float sdf_pentagon(vec2 pos, float sl) {
    const float d = 0.5 * sl * cos(radians(36));
    const vec3 k = vec3(0.809016994, 0.587785252, 0.726542528);

    pos.x = abs(pos.x);
    pos -= 2.0 * min(dot(vec2(-k.x, k.y), pos), 0.0) * vec2(-k.x, k.y);
    pos -= 2.0 * min(dot(vec2(k.x, k.y), pos), 0.0) * vec2(k.x, k.y);
    pos -= vec2(clamp(pos.x, -d * k.z, d * k.z), d);    
    return length(pos) * sign(-pos.y);
}

// sl(side length) ∈ [0, 1].
float sdf_hexagon(vec2 pos, float sl) {
    const float d = 0.5 * sl * cos(radians(30));
    const vec3 k = vec3(-0.866025404, 0.5, 0.577350269);
    
    pos = abs(pos);
    pos -= 2.0 * min(dot(k.xy, pos), 0.0) * k.xy;
    pos -= vec2(clamp(pos.x, -k.z * d, k.z * d), d);
    return length(pos) * sign(-pos.y);
}

// sl(side length) ∈ [0, 1].
float sdf_octogon(vec2 pos, float sl) {
    const float d = 0.5 * sl * cos(radians(22.5));
    const vec3 k = vec3(-0.9238795325, 0.3826834323, 0.4142135623 );
    
    pos = abs(pos);
    pos -= 2.0 * min(dot(vec2(k.x, k.y), pos), 0.0) * vec2(k.x, k.y);
    pos -= 2.0 * min(dot(vec2(-k.x, k.y), pos), 0.0) * vec2(-k.x,k.y);
    pos -= vec2(clamp(pos.x, -k.z * d, k.z * d), d);
    return length(pos) * sign(-pos.y);
}

// sl(side length) ∈ [0, 1].
float sdf_hexagram(vec2 pos, float sl) {
    const float d = 0.25 * sl;
    const vec4 k = vec4(-0.5, 0.8660254038, 0.5773502692, 1.7320508076);

    pos = abs(pos);
    pos -= 2.0 * min(dot(k.xy, pos), 0.0) * k.xy;
    pos -= 2.0 * min(dot(k.yx, pos), 0.0) * k.yx;
    pos -= vec2(clamp(pos.x, k.z * d,k.w * d), d);
    return length(pos) * sign(-pos.y);
}

// sl(side length) ∈ [0, 1].
float sdf_starfive(vec2 pos, float sl) {
    const float d = 0.5 * sl;
    
    const float an = 3.141593 / float(5);
    const float en = 3.141593 / 3.0;
    const vec2 acs = vec2(cos(an), sin(an));
    const vec2 ecs = vec2(cos(en), sin(en));

    float bn = mod(atan(pos.x, pos.y), 2.0 * an) - an;
    pos = length(pos) * vec2(cos(bn), abs(sin(bn)));

    pos -= d * acs;
    pos += ecs * clamp(-dot(pos, ecs), 0.0, d * acs.y / ecs.y);
    return length(pos) * sign(-pos.x);
}

float dot2( in vec2 v ) { return dot(v,v); }

// sl(side length) ∈ [0, 1].
// NOTE: 抄 + 瞎调参, 不懂原理; 看着能用, 可能有错!
float sdf_heart(vec2 pos, float sl) {
    pos *= 1.2 / sl;
    pos.y += 0.6;
    pos.x = abs(pos.x);

    if(pos.y + pos.x > 1.0)
        return -sqrt(dot2(pos - vec2(0.25, 0.75))) + sqrt(2.0) / 4.0;
    return sqrt(min(dot2(pos - vec2(0.00, 1.00)), dot2(pos - 0.5 * max(pos.x + pos.y, 0.0)))) * sign(-pos.x + pos.y);
}

void main() {
    uint gtype = types.x;
    vec2 pos_l = (mx_s2l * gl_FragCoord).xy;
    
    switch(gtype) {
        case GT_CIRCLE:
            o_Target = sign(sdf_circle(pos_l, 1.0)) * icolor;
            break;
        case GT_LINE:
            o_Target = icolor;
            break;
        case GT_ETRIANGLE:
            o_Target = sign(sdf_etriangle(pos_l, 1.0)) * icolor;
            break;
        case GT_SQUARE:
            o_Target = sign(sdf_square(pos_l, 1.0)) * icolor;
            break;
        case GT_PENTAGON:
            o_Target = sign(sdf_pentagon(pos_l, 1.0)) * icolor;
            break;
        case GT_HEXAGON:
            o_Target = sign(sdf_hexagon(pos_l, 1.0)) * icolor;
            break;
        case GT_OCTOGON:
            o_Target = sign(sdf_octogon(pos_l, 1.0)) * icolor;
            break;
        case GT_HEXAGRAM:
            o_Target = sign(sdf_hexagram(pos_l, 1.0)) * icolor;
            break;
        case GT_STARFIVE:
            o_Target = sign(sdf_starfive(pos_l, 1.0)) * icolor;
            break;
        case GT_HEART:
            o_Target = sign(sdf_heart(pos_l, 1.0)) * icolor;
            break;
        default:
            o_Target = vec4(1.0, 0.0, 1.0, 1.0);
            break;
    }
}