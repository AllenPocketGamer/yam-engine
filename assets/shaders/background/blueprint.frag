#version 450

// NOTE: MACROS

#define PI 3.14159265358979323846

// NOTE: CONSTANTS AREA

const vec4 ROYAL_BLUE       = vec4(48, 87, 225, 255) / 255.0;
const vec4 LAVENDER_BLUE    = vec4(206, 216, 247, 255) / 255.0;
const vec4 RESOLUTION_BLUE  = vec4(0, 32, 130, 255) / 255.0;

// The size of grid in `world space`.
const float GRID_SIZE               = 100.0;
// Size in `screen space`, when grid_size less than it, the level grid will disappear.
const float GRID_DISAPPEAR_SIZE     = 200.0;
const float GRID_SPLIT_COUNT        = 5.0;
// The thickness of grid in `screen space`.
const float GRID_THICKNESS          = 2.0;

// NOTE: BUFFERS AREA

layout(binding = 0) uniform Common {
    // Transform point from `world space` to `eye space`.
    mat4 MX_VIEW;
    // Transform point from `eye space` to `NDC`.
    mat4 MX_PROJECTION;
    // Transform point from `NDC` to `screen space`.
    mat4 MX_VIEWPORT;

    // Viewport size
    vec2 vp_size;

    // Delta time
    float t_delta;
    // Total time
    float t_total;
};

// NOTE: OUT VARIABLES

layout(location = 0) out vec4 o_Target;

// NOTE: FUNCTIONS AREA

float rand(float n){return fract(sin(n) * 43758.5453123);}

float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

float noise(float p){
	float fl = floor(p);
    float fc = fract(p);
	return mix(rand(fl), rand(fl + 1.0), fc);
}
	
float noise(vec2 n) {
    const vec2 d = vec2(0.0, 1.0);
    vec2 b = floor(n), f = smoothstep(vec2(0.0), vec2(1.0), fract(n));
    return mix(mix(rand(b), rand(b + d.yx), f.x), mix(rand(b + d.xy), rand(b + d.yy), f.x), f.y);
}

float noise(vec2 p, float freq){
    float unit = vp_size.x / freq;
    vec2 ij = floor(p / unit);
    vec2 xy = mod(p, unit) / unit;
    //xy = 3.*xy*xy-2.*xy*xy*xy;
    xy = 0.5 * (1.0 - cos(PI * xy));
    float a = rand((ij + vec2(0.0, 0.0)));
    float b = rand((ij + vec2(1.0 ,0.0)));
    float c = rand((ij + vec2(0.0, 1.0)));
    float d = rand((ij + vec2(1.0, 1.0)));
    float x1 = mix(a, b, xy.x);
    float x2 = mix(c, d, xy.x);
    return mix(x1, x2, xy.y);
}

float log_b(float base, float x) {
    return log(x) / log(base);
}

void main() {
    #define IN_GRID(uv, gs, gth) step(0.5 * vec2(gs) - abs(mod(uv, vec2(gs)) - 0.5 * vec2(gs)), vec2(gth))
    
    vec2 ps = gl_FragCoord.xy + vec2(0.5);
    vec2 cs = vp_size / 2.0;

    vec2 pw = (inverse(MX_VIEWPORT * MX_PROJECTION * MX_VIEW) * gl_FragCoord).xy;
    vec2 cw = vec2(0.0);

    // grid thickness half in `world space`.
    const float gth = 0.5 * length(inverse(MX_VIEWPORT * MX_PROJECTION * MX_VIEW) * vec4(GRID_THICKNESS, 0., 0., 0.));
    // The grid count be occupied by the disappear size.
    const float dc = GRID_DISAPPEAR_SIZE * gth / GRID_THICKNESS / GRID_SIZE;

    // The grid layer of dc.
    const float layer_dc = max(1.0, log_b(GRID_SPLIT_COUNT, dc) + 1.0);
    // The layer of the main grid.
    const float layer_main = ceil(layer_dc);
    // The layer of the sub grid.
    const float layer_sub = max(1.0, layer_main - 1.0);
    // The size of the main grid.
    const float grid_size_main = pow(GRID_SPLIT_COUNT, layer_main - 1.0) * GRID_SIZE;
    // The size of the sub grid.
    const float grid_size_sub = pow(GRID_SPLIT_COUNT, layer_sub - 1.0) * GRID_SIZE;

    // NOTE: 当缩放摄像机时, 在Grid会有移动, 观感稍微有点差, 但影响不大.
    const float n = noise(pw, 1000.0 / dc);

    // create the background grid in `world space`.
    const vec2 grid_uv = pw - cw;

    const float progress = fract(layer_dc);
    const float grid_main = dot(IN_GRID(grid_uv, grid_size_main, gth), vec2(1.0));
    const float grid_sub = dot(IN_GRID(grid_uv, grid_size_sub, gth), 0.5 * vec2(1.0 - progress));
    // const float grid = (grid_main + grid_sub) * smoothstep(0.4, 0.6, n);
    const float grid = (grid_main + grid_sub) * n;

    // background
    o_Target = clamp(ROYAL_BLUE + vec4(vec3(grid), 0.0), 0.0, 1.0);

    // grain
    // o_Target.rgb += n * 0.10;
    // o_Target.rgb = clamp(o_Target.rgb, 0.0, 1.0);

    o_Target.rgb *= (1.0 - length(cs - ps) / vp_size.x);
}