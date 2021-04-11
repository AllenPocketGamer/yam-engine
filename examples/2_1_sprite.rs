use yam::legion::{systems::CommandBuffer, *};
use yam::nalgebra::Vector2;
use yam::*;

fn main() -> Result<(), AppBuildError> {
    AppBuilder::new()
        .create_stage_builder(String::from("default"))?
        .add_thread_local_system_startup(introduction_system())
        .add_thread_local_system_startup(init_entities_system())
        .add_thread_local_system_process(control_camera_system())
        .add_thread_local_system_process(control_sprite_system())
        .into_app_builder()
        .build()
        .run();

    Ok(())
}

#[system]
fn introduction() {
    println!("Introduction:");
    println!("  1. Pressed the middle button of mouse to move the camera.");
    println!("  2. Pressed AWSD to move the sprite.");
}

#[system]
fn init_entities(commands: &mut CommandBuffer, #[resource] window: &Window) {
    let (width, height) = window.resolution();

    // Push camera entity to `World`.
    commands.push((Transform2D::default(), Camera2D::new(width, height)));
    // Push sprite entity to `World`.
    commands.push((
        Transform2D::with_scale(64.0, 64.0),
        Sprite { color: Rgba::RED },
    ));
}

#[system(for_each)]
#[filter(component::<Camera2D>())]
fn control_camera(
    transform2d: &mut Transform2D,
    #[resource] input: &Input,
    #[resource] time: &Time,
) {
    const MOVE_SPEED: f32 = 16.0;

    if input.mouse.pressed(MouseButton::Middle) {
        let (mx, my) = input.mouse.mouse_motion();
        transform2d.position += Vector2::new(mx, my) * time.delta().as_secs_f32() * MOVE_SPEED;
    }
}

#[system(for_each)]
#[filter(component::<Sprite>())]
fn control_sprite(
    transform2d: &mut Transform2D,
    #[resource] input: &Input,
    #[resource] time: &Time,
) {
    const MOVE_SPEED: f32 = 256.0;

    if input.keyboard.pressed(KeyCode::A) {
        transform2d.position -= Vector2::new(1.0, 0.0) * time.delta().as_secs_f32() * MOVE_SPEED;
    } else if input.keyboard.pressed(KeyCode::D) {
        transform2d.position += Vector2::new(1.0, 0.0) * time.delta().as_secs_f32() * MOVE_SPEED;
    }

    if input.keyboard.pressed(KeyCode::S) {
        transform2d.position -= Vector2::new(0.0, 1.0) * time.delta().as_secs_f32() * MOVE_SPEED;
    } else if input.keyboard.pressed(KeyCode::W) {
        transform2d.position += Vector2::new(0.0, 1.0) * time.delta().as_secs_f32() * MOVE_SPEED;
    }
}
