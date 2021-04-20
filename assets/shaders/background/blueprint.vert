#version 450

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

// NOTE: IN VARIABLES

// vertex
layout(location = 0) in vec4 v_pos;

// NOTE: FUNCTIONS AREA

mat4 to_matrix(vec2 position, vec2 complex, vec2 scale) {
    return mat4(
        complex.x * scale.x, complex.y * scale.x, 0.0, 0.0,     // column 0
        -complex.y * scale.y, complex.x * scale.y, 0.0, 0.0,    // column 1
        0.0, 0.0, 1.0, 0.0,                                     // column 2
        position.x, position.y, 0.0, 1.0                        // column 3
    );
}

void main() {
    gl_Position = vec4(2.0 * v_pos.xy, 0.0, 1.0);
}