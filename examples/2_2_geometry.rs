use yam::legion::{systems::CommandBuffer, *};
use yam::nalgebra::Vector2;
use yam::*;

fn main() -> Result<(), AppBuildError> {
    AppBuilder::new()
        .create_stage_builder(String::from("default"))?
        .add_thread_local_system_startup(init_entities_system())
        .add_thread_local_system_process(control_camera_system())
        .add_thread_local_system_process(control_geometry_tmp_system())
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

    let orange_circle = Geometry::circle_with_style(
        Vector2::new(0.0, 0.0),
        8.0,
        0,
        BorderType::Solid,
        Rgba::WHITE,
        0.1,
        InnerType::Solid,
        Rgba::ORANGE,
    );
    let chartreuse_etriangle = Geometry::triangle_with_style(
        Vector2::new(0.0, 0.0),
        8.0,
        0.0,
        0,
        BorderType::Solid,
        Rgba::WHITE,
        0.1,
        InnerType::Solid,
        Rgba::CHARTREUSE,
    );
    let spring_square = Geometry::square_with_style(
        Vector2::new(0.0, 0.0),
        8.0,
        0.0,
        0,
        BorderType::Solid,
        Rgba::WHITE,
        0.1,
        InnerType::Solid,
        Rgba::SPRING,
    );
    let azure_circle = Geometry::circle_with_style(
        Vector2::new(0.0, 0.0),
        8.0,
        0,
        BorderType::Solid,
        Rgba::WHITE,
        0.1,
        InnerType::Solid,
        Rgba::AZURE,
    );

    // 1-1 mode.
    cmd.push((Transform2D::new(0.0, 0.0, 0.0, 1.0, 1.0), orange_circle));
    cmd.push((
        Transform2D::new(32.0, 0.0, 0.0, 1.0, 1.0),
        chartreuse_etriangle,
    ));
    cmd.push((
        Transform2D::new(64.0, 0.0, 0.0, 1.0, 1.0),
        spring_square,
    ));

    // 1-N mode.
    cmd.push((
        Transform2D::new(0.0, 32.0, 0.0, 1.0, 1.0),
        vec![
            azure_circle,
            chartreuse_etriangle,
        ],
    ));

    // N-1 mode.
    cmd.push((
        vec![
            Transform2D::new(0.0, 64.0, 0.0, 1.0, 1.0),
            Transform2D::new(32.0, 64.0, 0.0, 1.0, 1.0),
        ],
        spring_square,
    ));

    // N-N mode.
    cmd.push((
        vec![
            Transform2D::new(0.0, 96.0, 0.0, 1.0, 1.0),
            Transform2D::new(32.0, 96.0, 0.0, 1.0, 1.0),
        ],
        vec![
            orange_circle,
            Geometry::new_circle(Vector2::new(16.0, 16.0), 8.0, 0),
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

#[system(for_each)]
#[filter(component::<Geometry>())]
fn control_geometry_tmp(
    transform: &mut Transform2D,
    #[resource] input: &Input,
    #[resource] time: &Time,
) {
    const TSPEED: f32 = 48.0;

    if input.keyboard.pressed(KeyCode::A) {
        transform.position.x -= TSPEED * time.delta().as_secs_f32();
    } else if input.keyboard.pressed(KeyCode::D) {
        transform.position.x += TSPEED * time.delta().as_secs_f32();
    }

    if input.keyboard.pressed(KeyCode::S) {
        transform.position.y -= TSPEED * time.delta().as_secs_f32();
    } else if input.keyboard.pressed(KeyCode::W) {
        transform.position.y += TSPEED * time.delta().as_secs_f32();
    }
}
