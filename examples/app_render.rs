// FIXME: temp
extern crate nalgebra as na;

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
        transform.rotate(RSPEED * time.delta().as_secs_f32());
    }
}

#[system(for_each)]
#[filter(component::<Camera2D>())]
fn operate_camera(transform: &mut Transform2D, #[resource] input: &Input) {
    if input.mouse.pressed(MouseButton::Middle) {
        let (dx, dy) = input.mouse.mouse_motion();

        transform.position += na::Vector2::<f32>::new(dx, -dy);
    }

    let (_, motion) = input.mouse.mouse_wheel_motion();
    transform.scale += na::Vector2::new(motion, motion);
}

fn init_entities(world: &mut World, _resources: &mut Resources) {
    // scale sprite size to 32
    world.push((
        Transform2D::new(0.0, 0.0, 0.0, 2560.0, 2560.0),
        Sprite { color: Color::RED },
    ));
    world.push((
        Transform2D::new(0.0, 0.0, 0.5, 32.0, 32.0),
        Sprite {
            color: Color::GREEN,
        },
        Marker {},
    ));
    world.push((Transform2D::default(), Camera2D::new(1920, 1080)));

    let mut transform2ds = Vec::<Transform2D>::with_capacity(10000);

    for x in 0..100 {
        for y in 0..100 {
            let (tx, ty) = (64.0 * x as f32, 64.0 * y as f32);

            transform2ds.push(Transform2D::new(tx, ty, 0.0, 32.0, 32.0));
        }
    }

    world.push((transform2ds, Sprite {color: Color::BLUE}));
}

struct Marker;