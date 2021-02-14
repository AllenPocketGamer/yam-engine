// FIXME: temp
extern crate nalgebra as na;

use legion::{query::*, Resources, World};
use yamengine::app::*;
use yamengine::input::*;
use yamengine::misc::*;
use yamengine::render::components::*;
use yamengine::*;

fn main() -> Result<(), AppBuildError> {
    AppBuilder::new()
        .create_stage_builder(String::from("default"))?
        .add_thread_local_fn_startup(init_entities)
        .add_thread_local_system_process(operate_sprite_system())
        .add_system_startup(parallel_startup_system())
        .add_system_destroy(parallel_destroy_system())
        .into_app_builder()
        .build()
        .run();

    Ok(())
}

#[system]
fn parallel_startup() {
    println!("parallel startup");
}

#[system]
fn parallel_destroy() {
    println!("parallel destroy");
}

#[system(for_each)]
#[filter(component::<Sprite>())]
fn operate_sprite(
    transform: &mut Transform2D,
    #[resource] time: &Time,
    #[resource] input: &Input,
) {
    let speed = 160f32;

    if input.keyboard.pressed(KeyCode::A) {
        transform.position -= na::Vector2::<f32>::x() * time.delta().as_secs_f32() * speed;
    } else if input.keyboard.pressed(KeyCode::D) {
        transform.position += na::Vector2::<f32>::x() * time.delta().as_secs_f32() * speed;
    }

    if input.keyboard.pressed(KeyCode::W) {
        transform.position += na::Vector2::<f32>::y() * time.delta().as_secs_f32() * speed;
    } else if input.keyboard.pressed(KeyCode::S) {
        transform.position -= na::Vector2::<f32>::y() * time.delta().as_secs_f32() * speed;
    }
}

fn init_entities(world: &mut World, _resources: &mut Resources) {
    world.push((Transform2D::default(), Sprite {}));
    world.push((Transform2D::default(), Camera2D::default()));
}
