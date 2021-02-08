[[location(0)]]
var<in> in_position: vec4<f32>;
[[builtin(position)]]
var<out> out_position: vec4<f32>;

[[block]]
struct Locals {
    transform: mat4x4<f32>;
};
[[group(0), binding(0)]]
var r_locals: Locals;

[[stage(vertex)]]
fn vs_main() {
    out_position = r_locals.transform * in_position;
    // out_position = in_position;
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
