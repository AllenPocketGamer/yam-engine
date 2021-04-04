#version 450

// NOTE: MACROS
#define PI 3.14159265359

// NOTE: CONSTANTS AREA

// The ratio between dash_length : thickness.
const float DASH_PROPORTION = 8.0;
// The empty dash proportion.
const float DASH_EMPTY = 0.3;
// Anti-aliasing pixel count.
const float BLUR = 1.0;

// TEMP
const float TEMP_THICKNESS = 0.02;
const float TEMP_HT = 0.5 * TEMP_THICKNESS;

// GeometryType
const uint GT_CIRCLE        = 0;
const uint GT_LINE          = 1;
const uint GT_ETRIANGLE     = 2;
const uint GT_SQUARE        = 3;
const uint GT_PENTAGON      = 4;
const uint GT_HEXAGON       = 5;
const uint GT_OCTOGON       = 6;
const uint GT_HEXAGRAM      = 7;
const uint GT_STARFIVE      = 8;
const uint GT_HEART         = 9;

// BorderType
const uint BT_NONE          = 0;
const uint BT_SOLID         = 1;
const uint BT_DASH          = 2;
const uint BT_DYN_DASH      = 3;
const uint BT_NAVI          = 4;
const uint BT_DYN_NAVI      = 5;
const uint BT_WARN          = 6;
const uint BT_DYN_WARN      = 7;

// InnerType
const uint IT_NONE          = 0;
const uint IT_SOLID         = 1;
const uint IT_DITHER        = 2;
const uint IT_DYN_DITHER    = 3;

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
    // Transform point from `world space` to `eye space`.
    mat4 MX_VIEW;
    // Transform point from `eye space` to `NDC`.
    mat4 MX_PROJECTION;
    // Transform point from `NDC` to `screen space`.
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
flat layout(location = 4) in mat4 mx_g2l;
flat layout(location = 8) in mat4 mx_l2w;

// NOTE: OUT VARIABLES

layout(location = 0) out vec4 o_Target;

// NOTE: FUNCTIONS AREA

// SDF函数可以参考这里: https://www.iquilezles.org/www/articles/distfunctions2d/distfunctions2d.htm

// sl(side length) ∈ [0, 1].
float sdf_circle(vec2 pos, float sl) {
    const float radius = 0.5 * sl;
    
    return radius - length(pos);
}

// sl(side length) ∈ [0, 1].
float sdf_etriangle(vec2 pos, float sl) {
    const float k = sqrt(3.0);

    // The radius of the circle enclosing the etriangle.
    const float radius = 0.5 * sl;
    // the side length of etriangle.
    const float etsl = k * radius;

    // Map points along the Y axis.
    pos.x = abs(pos.x) - 0.5 * etsl;
    pos.y = pos.y + 0.5 * radius;
    // Map points along the `l: x + √3y = 0` axis.
    pos = pos.x + k * pos.y > 0.0 ? vec2(pos.x - k * pos.y, -k * pos.x - pos.y) / 2.0 : pos;
    
    // // 圆角
    // pos.x -= clamp(pos.x, -etsl, 0.0);
    // return length(pos) * sign(pos.y);

    // 无圆角
    return pos.y;
}

// // sl(side length) ∈ [0, 1].
// float sdf_square(vec2 pos, float sl) {
//     pos = abs(pos);
//     pos = pos.x - pos.y > 0.0 ? pos = pos.yx : pos;
//     pos.y -= 0.5 * sl;

//     pos.x -= clamp(pos.x, 0, 0.5 * sl);
    
//     return length(pos) * sign(-pos.y);
// }

// sl(side length) ∈ [0, 1].
float sdf_square(vec2 pos, float sl) {
   pos = abs(pos);
   return min(0.5 - pos.x, 0.5 - pos.y);
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
    
    const float an = PI / float(5);
    const float en = PI / 3.0;
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

float get_circle_dash(
    const vec2 pg,
    const float blur_g,
    const float time
) {
    // radius in `geometry space`.
    const float rad = atan(pg.y, pg.x);
    // [-PI, PI] --map-> [-1, 1]. 
    const float maped = rad / PI;
    const float count = 2.0 * ceil(0.5 * PI / (DASH_PROPORTION * TEMP_THICKNESS));

    // blur_r是blur_g在dash弧度长上的占比.
    const float blur_r = blur_g * count / PI;

    return smoothstep(DASH_EMPTY - blur_r, DASH_EMPTY + blur_r, 
        abs(fract((maped - 0.5) * count / 4.0 + time) - 0.5) * 2.0);
}

// Returns vec2(inner, decoration).
vec2 get_inner(
    const uint itype,
    const vec2 pg,
    const float sdf,
    const float blur_g,
    const float hth_g,
    const float time
) {
    float inner = 0.0;
    float decoration = 0.0;

    switch(itype) {
        case IT_NONE:
            break;
        case IT_SOLID:
            inner = smoothstep(-blur_g, blur_g, sdf - hth_g);
            decoration = 1.0;
            break;
        case IT_DITHER:
            // TODO
            break;
        case IT_DYN_DITHER:
            // TODO
            break;
        default:
            break;
    }

    return vec2(inner, decoration);
}

// Returns vec2(border, decoration).
vec2 get_border(
    const uint btype,
    const vec2 pg,
    const float sdf,
    const float blur_g,
    const float hth_g,
    const float time
) {
    float border = 0.0;
    float decoration = 0.0;
    
    switch(btype) {
        case BT_NONE:
            break;
        case BT_SOLID:
            border = 1.0 - smoothstep(hth_g - blur_g, hth_g + blur_g, abs(sdf - hth_g));
            decoration = 1.0;
            break;
        case BT_DASH:
            border = 1.0 - smoothstep(hth_g - blur_g, hth_g + blur_g, abs(sdf - hth_g));
            decoration = get_circle_dash(pg, blur_g, 0.0);
            break;
        case BT_DYN_DASH:
            border = 1.0 - smoothstep(hth_g - blur_g, hth_g + blur_g, abs(sdf - hth_g));
            decoration = get_circle_dash(pg, blur_g, time);
            break;
        case BT_NAVI:
            // TODO
            break;
        case BT_DYN_NAVI:
            // TODO
            break;
        case BT_WARN:
            // TODO
            break;
        case BT_DYN_WARN:
            // TODO
            break;
        default:
            break;
    }

    return vec2(border, decoration);
}

void main() {
    // Transform points from `screen space` to `geometry space`.
    const mat4 mx_g2s = MX_VIEWPORT * MX_PROJECTION * MX_VIEW * mx_l2w * mx_g2l;
    // Transform points from `screen space` to `geometry space`.
    const mat4 mx_s2g = inverse(mx_g2s);
    
    // frag coordinate without the order info of geometry in `screen space`.
    const vec4 ps = vec4(gl_FragCoord.xy, 0.0, 1.0);
    // frag coordniate in `geometry space`.
    //
    // careful the pg rewrites the z component to remove the order info of geometry.
    const vec4 pg = vec4((mx_s2g * ps).xy, 0.0, 1.0);
    // quad centra in `screen space`.
    //
    // careful the cs rewrites the z component to remove the order info of geometry.
    const vec4 cs = vec4((mx_g2s * vec4(0.0, 0.0, 0.0, 1.0)).xy, 0.0, 1.0);

    // vector from quad centra to frag in screen space.
    const vec4 avoid_zero = vec4(0.00001, 0.00001, 0.0, 0.0);
    const vec4 c2p_norm_s = normalize(ps + avoid_zero - cs);

    // blur factor in `geometry space`.
    const float blur_g = BLUR * length(mx_s2g * c2p_norm_s);

    const uint gtype = types.x;
    const uint btype = types.y;
    const uint itype = types.z;
    
    float sdf;

    switch(gtype) {
        case GT_CIRCLE:
            sdf = sdf_circle(pg.xy, 1.0);
            break;
        case GT_LINE:
            o_Target = icolor;
            break;
        case GT_ETRIANGLE:
            sdf = sdf_etriangle(pg.xy, 1.0);
            break;
        case GT_SQUARE:
            sdf = sdf_square(pg.xy, 1.0);
            break;
        case GT_PENTAGON:
            sdf = sdf_pentagon(pg.xy, 1.0);
            break;
        case GT_HEXAGON:
            sdf = sdf_hexagon(pg.xy, 1.0);
            break;
        case GT_OCTOGON:
            sdf = sdf_octogon(pg.xy, 1.0);
            break;
        case GT_HEXAGRAM:
            sdf = sdf_hexagram(pg.xy, 1.0);
            break;
        case GT_STARFIVE:
            sdf = sdf_starfive(pg.xy, 1.0);
            break;
        case GT_HEART:
            sdf = sdf_heart(pg.xy, 1.0);
            break;
        default:
            break;
    }

    const vec2 inner = get_inner(itype, pg.xy, sdf, blur_g, TEMP_HT, 0.0);
    const vec2 border = get_border(btype, pg.xy, sdf, blur_g, TEMP_HT, 0.0);
    o_Target = mix(inner.x * icolor, border.y * bcolor, border.x);
}