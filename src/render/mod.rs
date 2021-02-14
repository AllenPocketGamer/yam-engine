pub mod components;
mod renderer;

extern crate nalgebra as na;

use crate::{
    app::{AppStage, AppStageBuilder},
    window::Window,
};
use components::*;
use futures::executor::block_on;
use legion::{query::*, Resources, World};

pub(crate) fn create_app_stage_render() -> AppStage {
    AppStageBuilder::new(String::from("default_render"))
        .add_thread_local_fn_startup(init)
        // .add_thread_local_fn_process(temp_render)
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
    // NOTE: render sprite
    let mut gpu = resources.get_mut::<renderer::Gpu>().unwrap();
    let mut sprite_renderer = resources.get_mut::<renderer::SpriteRenderer>().unwrap();

    let mut query_camera2d = <(&Transform2D, &Camera2D)>::query();
    let mut query_sprites = <(&Transform2D, &Sprite)>::query();

    let (transform_camera2d, camera2d) = query_camera2d.iter(world).next().unwrap();
    let (transform_sprite, _) = query_sprites.iter(world).next().unwrap();

    let mx_model = transform_sprite.to_homogeneous_3d();
    let mx_view = transform_camera2d.to_homogeneous_3d().try_inverse().unwrap();
    let mx_projection = camera2d.to_homogeneous();
    
    sprite_renderer.set_transformations(&mut gpu, &mx_model, &mx_view, &mx_projection);
    sprite_renderer.render(&mut gpu);
}
