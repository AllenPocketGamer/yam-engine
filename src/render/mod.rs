pub mod components;
mod renderer;

extern crate nalgebra as na;

use crate::{app::{AppStage, AppStageBuilder}, misc::Color, window::Window};
use components::*;
use futures::executor::block_on;
use legion::{query::*, Resources, World};

pub(crate) fn create_app_stage_render() -> AppStage {
    AppStageBuilder::new(String::from("default_render"))
        .add_thread_local_fn_startup(init)
        .add_thread_local_fn_process(render)
        .build()
}

fn init(_world: &mut World, resources: &mut Resources) {
    let mut gpu = {
        let window = resources
            .get::<Window>()
            .expect("not found resource window.");
        block_on(renderer::Gpu::new(&window.window))
    };
    let sprite_renderer = renderer::SpriteRenderer::new(&mut gpu);

    resources.insert(gpu);
    resources.insert(sprite_renderer);
}

fn render(world: &mut World, resources: &mut Resources) {
    let window = resources.get::<Window>().unwrap();
    let mut gpu = resources.get_mut::<renderer::Gpu>().unwrap();
    let mut sprite_renderer = resources.get_mut::<renderer::SpriteRenderer>().unwrap();

    // FIXME: resize swapchain(temporary)
    gpu.set_swap_chain_size(window.inner_size());

    // NOTE: render sprite
    let mut query_camera2d = <(&Transform2D, &Camera2D)>::query();
    let mut query_sprites = <(&Transform2D, &Sprite)>::query();

    let (mx_view, mx_projection) =
        if let Some((transform, camera2d)) = query_camera2d.iter(world).next() {
            (
                transform.to_homogeneous_3d().try_inverse().unwrap(),
                camera2d.to_homogeneous(),
            )
        } else {
            (
                na::Matrix4::<f32>::identity(),
                Camera2D::default().to_homogeneous(),
            )
        };

    gpu.begin_render();

    sprite_renderer.clear(&mut gpu, Color::BLACK);

    for (transform_sprite, sprite) in query_sprites.iter(world) {
        let mx_model = transform_sprite.to_homogeneous_3d();
        sprite_renderer.render(&mut gpu, &mx_model, &mx_view, &mx_projection, sprite.color);
    }

    gpu.end_render();
}
