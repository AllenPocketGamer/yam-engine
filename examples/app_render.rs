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
        .add_thread_local_system_process(operate_camera_system())
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
#[filter(component::<Marker>())]
fn operate_sprite(transform: &mut Transform2D, #[resource] time: &Time, #[resource] input: &Input) {
    const SPEED: f32 = 160.0;
    const RSPEED: f32 = 3.14;

    if input.keyboard.pressed(KeyCode::A) {
        transform.position -= na::Vector2::<f32>::x() * time.delta().as_secs_f32() * SPEED;
    } else if input.keyboard.pressed(KeyCode::D) {
        transform.position += na::Vector2::<f32>::x() * time.delta().as_secs_f32() * SPEED;
    }

    if input.keyboard.pressed(KeyCode::W) {
        transform.position += na::Vector2::<f32>::y() * time.delta().as_secs_f32() * SPEED;
    } else if input.keyboard.pressed(KeyCode::S) {
        transform.position -= na::Vector2::<f32>::y() * time.delta().as_secs_f32() * SPEED;
    }

    if input.keyboard.pressed(KeyCode::Space) {
        transform.angle += RSPEED * time.delta().as_secs_f32();
    }
}

#[system(for_each)]
#[filter(component::<Camera2D>())]
fn operate_camera(transform: &mut Transform2D, #[resource] input: &Input) {
    if input.mouse.pressed(MouseButton::Middle) {
        let (dx, dy) = input.mouse.mouse_motion();

        transform.position += na::Vector2::<f32>::new(dx, -dy);
    }
}

fn init_entities(world: &mut World, _resources: &mut Resources) {
    // scale sprite size to 32
    world.push((
        Transform2D::new(0.0, 0.0, 0.0, 2560.0, 2560.0),
        Sprite { color: Color::RED },
    ));
    world.push((
        Transform2D::new(64.0, 0.0, 0.5, 32.0, 32.0),
        Sprite {
            color: Color::GREEN,
        },
        Marker {},
    ));
    world.push((Transform2D::default(), Camera2D::new(1920f32, 1080f32)));
}

struct Marker;