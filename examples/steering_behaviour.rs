use yam::legion::{systems::CommandBuffer, *};
use yam::nalgebra::Vector2;
use yam::*;

static RADIUS: f32 = 128.0;
static DISTANCE: f32 = 256.0;

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
fn init_entities(cmds: &mut CommandBuffer) {
    cmds.push((Transform2D::default(), Camera2D::default()));

    let wander_assembly = vec![
        // main
        Geometry::new_2d(
            Geometry2DType::ETriangle,
            BorderDecoration::Solid,
            Rgba::SOFT_BLACK,
            BorderThickness::LocalSpace(4.0),
            InnerDecoration::Solid,
            Rgba::ROSE,
            0,
            Vector2::new(0.0, 0.0),
            0.0,
            64.0,
        ),
        // wander circle
        Geometry::new_2d(
            Geometry2DType::Circle,
            BorderDecoration::Dash,
            Rgba::SOFT_BLACK,
            BorderThickness::LocalSpace(4.0),
            InnerDecoration::None,
            Rgba::default(),
            1,
            Vector2::new(0.0, DISTANCE),
            0.0,
            2.0 * RADIUS,
        ),
        // wander target
        Geometry::new_2d(
            Geometry2DType::Circle,
            BorderDecoration::Solid,
            Rgba::SOFT_BLACK,
            BorderThickness::LocalSpace(2.0),
            InnerDecoration::Solid,
            Rgba::CAMEL,
            2,
            Vector2::new(0.0, RADIUS + DISTANCE),
            0.0,
            16.0,
        ),
    ];
    cmds.push((Transform2D::default(), wander_assembly));
}

#[system(for_each)]
#[filter(component::<Camera2D>())]
fn control_camera(transform: &mut Transform2D, #[resource] input: &Input) {
    const SSPEED: f32 = 0.40;

    if input.mouse.pressed(MouseButton::Middle) {
        let (dx, dy) = input.mouse.mouse_motion_in_ws();

        transform.position -= Vector2::<f32>::new(dx, dy);
    }

    let (_, motion) = input.mouse.mouse_wheel_motion();
    transform.scale = Vector2::new(
        (transform.scale.x + motion * SSPEED).max(0.1),
        (transform.scale.y + motion * SSPEED).max(0.1),
    );
}