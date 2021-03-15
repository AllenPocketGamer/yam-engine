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

    // // 层级树支持: 并排容易支持, I型和II型串排支持得考虑应用场景.

    // // 1(串排). Push 2 circle to `World`.
    // cmd.push((
    //     Transform2D::with_position(0.0, 0.0),
    //     Geometry::with_color(
    //         GeometryType::Circle,
    //         Rgba::CYAN.to_hex(),
    //         Rgba::WHITE.to_hex(),
    //     ),
    // ));
    // cmd.push((
    //     Transform2D::with_position(4.0, 0.0),
    //     Geometry::with_color(
    //         GeometryType::Circle,
    //         Rgba::CYAN.to_hex(),
    //         Rgba::ORANGE.to_hex(),
    //     ),
    // ));

    // 2(串排). Push 2 square to `World`, euqal to 1.
    //
    // 串排模式会渲染<Transform2D + Sprite/Geometry>`N`次, N = Instance<..>.len().
    //
    // 支持串排模式:
    // [Transform2D + Sprite/Geometry + ...] + ...
    let squares: Instance<(Transform2D, Geometry)> = vec![
        (
            Transform2D::new(0.0, 4.0, 0.0, 16.0, 16.0),
            Geometry::circle_default_style(Vector2::new(0.0, 0.0), 2.0, 0),
        ),
        (
            Transform2D::new(128.0, 4.0, 0.0, 16.0, 64.0),
            Geometry::circle_default_style(Vector2::new(0.0, 0.0), 2.0, 0),
        ),
    ];
    cmd.push((squares,));

    // // 3(并排). Push 2 triangle to `World`.
    // //
    // // 并排模式会渲染相同的Sprite/Geometry`N`次, `N` = [Transform2D].len().
    // //
    // // 支持并排模式:
    // // [Transform2D] + Sprite/Geometry + ...
    // let tris: Instance<Transform2D> = vec![
    //     Transform2D::with_position(0.0, 8.0),
    //     Transform2D::with_position(4.0, 8.0),
    // ];
    // cmd.push((tris, Geometry::with_default_style(GeometryType::Triangle)));
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
