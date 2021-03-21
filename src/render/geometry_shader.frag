#version 450

layout(push_constant) uniform CONSTANTS {
    vec4 VIEWPORT;  // (x, y, w, h)
    mat4 MX_VIEW;
    mat4 MX_PROJECTION;
};

// NOTE: Transform point from NDC to screen space
//
// x_ss = (x_ndc + 1) / 2 * width + vp.x        , x_ndc ∈ [-1, 1]
// y_ss = (1 - y_ndc) / 2 * height + vp.z       , y_ndc ∈ [-1, 1]
// z_ss = (far - near) * z_ndc + near           , z_ndc ∈ [+0, 1]
mat4 mx_viewport() {
    float x = VIEWPORT.x;
    float y = VIEWPORT.y;
    float w = VIEWPORT.z;
    float h = VIEWPORT.w;
    
    return mat4(
        0.5 * w,    0.0,        0.0,    0.0,    // column 0
        0.0,        -0.5 * h,   0.0,    0.0,    // column 1
        0.0,        0.0,        1.0,    0.0,    // column 2
        0.5 * w + x,0.5 * h + y,0.0,    1.0     // column 3
    );
}

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

    vec2 centra_ss = (mx_viewport() * vec4(centra_ndc.xy, 0, 1)).xy;

    float radius_ws = tmp.z;
    vec4 r_vec_ndc = MX_PROJECTION * MX_VIEW * vec4(radius_ws, 0, 0, 0);
    float radius_ss = length(mx_viewport() * r_vec_ndc);

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