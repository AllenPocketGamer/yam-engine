pub mod components;
mod renderer;

extern crate nalgebra as na;

pub use components::*;

use crate::*;
use renderer::Render2DService;

pub(crate) fn create_app_stage_render(window: &winit::window::Window) -> AppStage {
    let mut render2d_service = Render2DService::new(window);

    let render_process = move |world: &mut World, resources: &mut Resources| {
        let (width, height) = {
            let window = resources
                .get::<Window>()
                .expect("ERR: Not find window resource.");
            window.inner_size()
        };

        let mut query_camera2d = <(&Transform2D, &Camera2D)>::query();
        let mut query_sprites = <(&Transform2D, &Sprite)>::query();

        render2d_service.set_swap_chain_size(width, height);

        if let Some((transform2d, camera2d)) = query_camera2d.iter(world).next() {
            render2d_service.set_view_transformation(transform2d);
            render2d_service.set_projection(camera2d);
            render2d_service.set_viewport_aspect_ratio(camera2d.aspect_ratio());
        }

        render2d_service.begin_draw();

        for (transform2d, sprite) in query_sprites.iter(world) {
            render2d_service.draw_sprite_in_world_space(transform2d, sprite);
        }

        render2d_service.end_draw();
    };

    AppStageBuilder::new(String::from("default_render"))
        .add_thread_local_fn_process(render_process)
        .build()
}