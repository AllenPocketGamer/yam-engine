pub mod components;
mod renderer;

extern crate nalgebra as na;

pub use components::*;

use crate::*;
use renderer::Render2DService;

pub(crate) fn create_app_stage_render(window: &winit::window::Window) -> AppStage {
    let mut r2ds = Render2DService::new(window);

    let render_process = move |world: &mut World, resources: &mut Resources| {
        let (width, height) = {
            let window = resources
                .get::<Window>()
                .expect("ERR: Not find window resource.");
            window.inner_size()
        };

        let mut query_camera2d = <(&Transform2D, &Camera2D)>::query();
        let mut query_sprites = <(&Transform2D, &Sprite)>::query();
        let mut query_sprites_instanced = <(&Vec<Transform2D>, &Sprite)>::query();

        r2ds.set_swap_chain_size(width, height);

        if let Some((transform2d, camera2d)) = query_camera2d.iter(world).next() {
            r2ds.set_view_transformation(transform2d);
            r2ds.set_projection(camera2d);
            r2ds.set_viewport_aspect_ratio(camera2d.aspect_ratio());
        }

        r2ds.begin_draw();

        for (transform2d, sprite) in query_sprites.iter(world) {
            r2ds.draw_sprite_in_world_space(transform2d, sprite);
        }

        for (transform2ds, sprite) in query_sprites_instanced.iter(world) {
            r2ds.draw_sprites_in_world_space(&transform2ds[..], sprite);
        }

        r2ds.finish_draw();
    };

    AppStageBuilder::new(String::from("default_render"))
        .add_thread_local_fn_process(render_process)
        .build()
}
