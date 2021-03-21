#version 450

layout(push_constant) uniform CONSTANTS {
    // Transform point from `World` space to `Eye` space.
    mat4 MX_VIEW;
    // Transform point from `Eye` space to `NDC`.
    mat4 MX_PROJECTION;
    // Transform point from `NDC` to `Screen` space.
    mat4 MX_VIEWPORT;
};

layout(location = 0) in vec4 v_color;
layout(location = 1) in vec4 v_tmp;

layout(location = 0) out vec4 o_Target;

float tex_circle(vec4 tmp) {
    if(tmp.w != 0) {
        return 1.0;
    }
    
    vec4 centra_wp = vec4(tmp.xy, 0, 1);
    vec4 centra_cp = MX_PROJECTION * MX_VIEW * centra_wp;
    vec2 centra_ndc = centra_cp.xy / centra_cp.w;

    vec2 centra_ss = (MX_VIEWPORT * vec4(centra_ndc.xy, 0, 1)).xy;

    float radius_ws = tmp.z;
    vec4 r_vec_ndc = MX_PROJECTION * MX_VIEW * vec4(radius_ws, 0, 0, 0);
    float radius_ss = length(MX_VIEWPORT * r_vec_ndc);

    float diff = distance(gl_FragCoord.xy, centra_ss);

    return sign(radius_ss - diff);
}

void main() {
    if(tex_circle(v_tmp) > 0) {
        o_Target = v_color;
    } else {
        discard;
    }
    // o_Target = v_color;
}