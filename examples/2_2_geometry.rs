use yam::legion::{systems::CommandBuffer, *};
use yam::nalgebra::Vector2;
use yam::*;

fn main() -> Result<(), AppBuildError> {
    AppBuilder::new()
        .create_stage_builder(String::from("default"))?
        .add_thread_local_system_startup(init_entities_system())
        .add_thread_local_system_process(control_camera_system())
        .into_app_builder()
        .build()
        .run();

    Ok(())
}

#[system]
fn init_entities(cmd: &mut CommandBuffer, #[resource] window: &Window) {
    let (width, height) = window.resolution();

    // Push camera entity to `World`.
    cmd.push((Transform2D::default(), Camera2D::new(width, height)));

    // 1-1 mode.
    cmd.push((
        Transform2D::new(0.0, 0.0, 0.0, 16.0, 16.0),
        Geometry::new_circle(Vector2::new(0.0, 0.0), 16.0, 0),
    ));
    cmd.push((
        Transform2D::new(32.0, 0.0, 0.0, 16.0, 16.0),
        Geometry::new_circle(Vector2::new(0.0, 0.0), 16.0, 0),
    ));
    cmd.push((
        Transform2D::new(64.0, 0.0, 0.0, 16.0, 16.0),
        Geometry::new_circle(Vector2::new(0.0, 0.0), 16.0, 0),
    ));

    // 1-N mode.
    cmd.push((
        Transform2D::new(0.0, 32.0, 0.0, 16.0, 16.0),
        vec![
            Geometry::new_triangle(Vector2::new(0.0, 0.0), 16.0, 0.0, 0),
            Geometry::new_triangle(Vector2::new(32.0, 0.0), 16.0, 0.0, 0),
        ],
    ));

    // N-1 mode.
    cmd.push((
        vec![
            Transform2D::new(0.0, 64.0, 0.0, 16.0, 16.0),
            Transform2D::new(32.0, 64.0, 0.0, 16.0, 16.0),
        ],
        Geometry::new_square(Vector2::new(0.0, 0.0), 16.0, 0.0, 0),
    ));

    // N-N mode.
    cmd.push((
        vec![
            Transform2D::new(0.0, 96.0, 0.0, 16.0, 16.0),
            Transform2D::new(32.0, 96.0, 0.0, 16.0, 16.0),
        ],
        vec![
            Geometry::new_circle(Vector2::new(0.0, 0.0), 16.0, 0),
            Geometry::new_circle(Vector2::new(16.0, 12.0), 24.0, 0),
        ],
    ));
}

#[system(for_each)]
#[filter(component::<Camera2D>())]
fn control_camera(transform: &mut Transform2D, #[resource] input: &Input) {
    const TSPEED: f32 = 4.0;
    const SSPEED: f32 = 0.40;

    if input.mouse.pressed(MouseButton::Middle) {
        let (dx, dy) = input.mouse.mouse_motion();

        transform.position += Vector2::<f32>::new(dx, -dy) * TSPEED;
    }

    let (_, motion) = input.mouse.mouse_wheel_motion();
    transform.scale = Vector2::new(
        (transform.scale.x + motion * SSPEED).max(0.2),
        (transform.scale.y + motion * SSPEED).max(0.2),
    );
}
