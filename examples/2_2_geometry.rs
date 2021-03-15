use yam::legion::{systems::CommandBuffer, *};
use yam::*;

fn main() -> Result<(), AppBuildError> {
    AppBuilder::new()
        .create_stage_builder(String::from("default"))?
        .add_thread_local_system_startup(init_entities_system())
        .into_app_builder()
        .build()
        .run();

    Ok(())
}

#[system]
fn init_entities(cmd: &mut CommandBuffer, #[resource] window: &Window) {
    // let (width, height) = window.resolution();

    // // Push camera entity to `World`.
    // cmd.push((Transform2D::default(), Camera2D::new(width, height)));

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

    // // 2(串排). Push 2 square to `World`, euqal to 1.
    // //
    // // 串排模式会渲染<Transform2D + Sprite/Geometry>`N`次, N = Instance<..>.len().
    // // 
    // // 支持串排模式:
    // // [Transform2D + Sprite/Geometry + ...] + ...
    // let squares: Instance<(Transform2D, Geometry)> = vec![
    //     (
    //         Transform2D::with_position(0.0, 4.0),
    //         Geometry::with_color(
    //             GeometryType::Square,
    //             Rgba::CYAN.to_hex(),
    //             Rgba::WHITE.to_hex(),
    //         ),
    //     ),
    //     (
    //         Transform2D::with_position(4.0, 4.0),
    //         Geometry::with_color(
    //             GeometryType::Square,
    //             Rgba::CYAN.to_hex(),
    //             Rgba::ORANGE.to_hex(),
    //         ),
    //     ),
    // ];
    // cmd.push((squares,));

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
