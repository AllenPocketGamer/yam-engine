// const MT_CORRECTION: mat4x4<f32> = mat4x4<f32>(
//     1.0, 0.0, 0.0, 0.0,
//     0.0, 1.0, 0.0, 0.0,
//     0.0, 0.0, 0.5, 0.5,
//     0.0, 0.0, 0.0, 1.0,
// );

[[location(0)]]
var<in> in_position: vec4<f32>;

[[builtin(position)]]
var<out> out_position: vec4<f32>;

[[block]]
struct Translations {
    mt_model: mat3x3<f32>;
    mt_view: mat3x3<f32>;
    mt_projection: mat4x4<f32>;
};
[[group(0), binding(0)]]
var trans: Translations;

[[stage(vertex)]]
fn vs_main() {
    // var mt_correction: mat4x4<f32> = mat4x4<f32>(
    //     1.0, 0.0, 0.0, 0.0,
    //     0.0, 1.0, 0.0, 0.0,
    //     0.0, 0.0, 0.5, 0.5,
    //     0.0, 0.0, 0.0, 1.0
    // );

    // var pos: vec3<f32> = vec3<f32>(in_position.x, in_position.y, 1.0);
    // var pos_ndc: vec3<f32> = trans.mt_model * trans.mt_view * pos;
    // out_position = mt_correction * vec4<f32>(pos_ndc.x, pos_ndc.y, 0.0, 1.0);
    out_position = vec4<f32>(in_position.x, in_position.y, 0.5, 1.0);
}

[[location(0)]]
var<out> out_color: vec4<f32>;

[[stage(fragment)]]
fn fs_main() {
    out_color = vec4<f32>(0.2, 0.5, 0.6, 1.0);
    //TODO: support `length` and `mix` functions
    //var mag: f32 = length(in_tex_coord_fs-vec2<f32>(0.5, 0.5));
    //out_color = vec4<f32>(mix(tex.xyz, vec3<f32>(0.0, 0.0, 0.0), mag*mag), 1.0);
}
