use yam::legion::{systems::CommandBuffer, *};
use yam::nalgebra::Vector2;
use yam::*;

fn main() -> Result<(), AppBuildError> {
    AppBuilder::new()
        .create_stage_builder(String::from("default"))?
        .add_thread_local_system_startup(init_entities_system())
        .add_thread_local_system_process(control_camera_system())
        .add_thread_local_system_process(control_geometry_system())
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

    let th_l = BorderThickness::LocalSpace(2.0);
    let size: f32 = 100.0;

    let (x0, x1, x2) = (-0.5 * size, 0.5 * size, 1.5 * size);

    cmd.push((
        Transform2D::with_position(x0, x0),
        Geometry::new_2d(
            Geometry2DType::Circle,
            BorderDecoration::DynDash,
            Rgba::SOFT_BLACK,
            th_l,
            InnerDecoration::Solid,
            Rgba::ORANGE,
            100,
            Vector2::new(0.0, 0.0),
            0.0,
            size,
        ),
    ));

    cmd.push((
        Transform2D::with_position(x1, x0),
        Geometry::new_2d(
            Geometry2DType::ETriangle,
            BorderDecoration::Solid,
            Rgba::SOFT_BLACK,
            th_l,
            InnerDecoration::Solid,
            Rgba::YELLOW,
            100,
            Vector2::new(0.0, 0.0),
            0.0,
            size,
        ),
    ));

    cmd.push((
        Transform2D::with_position(x2, x0),
        Geometry::new_2d(
            Geometry2DType::Square,
            BorderDecoration::DynDash,
            Rgba::SOFT_BLACK,
            th_l,
            InnerDecoration::Solid,
            Rgba::CHARTREUSE,
            100,
            Vector2::new(0.0, 0.0),
            0.0,
            size,
        ),
    ));

    cmd.push((
        Transform2D::with_position(x0, x1),
        Geometry::new_2d(
            Geometry2DType::Pentagon,
            BorderDecoration::Solid,
            Rgba::SOFT_BLACK,
            th_l,
            InnerDecoration::Solid,
            Rgba::SPRING,
            100,
            Vector2::new(0.0, 0.0),
            0.0,
            size,
        ),
    ));

    cmd.push((
        Transform2D::with_position(x1, x1),
        Geometry::new_2d(
            Geometry2DType::Hexagon,
            BorderDecoration::DynDash,
            Rgba::SOFT_BLACK,
            th_l,
            InnerDecoration::Solid,
            Rgba::CYAN,
            100,
            Vector2::new(0.0, 0.0),
            0.0,
            size,
        ),
    ));

    cmd.push((
        Transform2D::with_position(x2, x1),
        Geometry::new_2d(
            Geometry2DType::Octogon,
            BorderDecoration::Solid,
            Rgba::SOFT_BLACK,
            th_l,
            InnerDecoration::Solid,
            Rgba::AZURE,
            100,
            Vector2::new(0.0, 0.0),
            0.0,
            size,
        ),
    ));

    cmd.push((
        Transform2D::with_position(x0, x2),
        Geometry::new_2d(
            Geometry2DType::Hexagram,
            BorderDecoration::DynDash,
            Rgba::SOFT_BLACK,
            th_l,
            InnerDecoration::Solid,
            Rgba::VIOLET,
            100,
            Vector2::new(0.0, 0.0),
            0.0,
            size,
        ),
    ));

    cmd.push((
        Transform2D::with_position(x1, x2),
        Geometry::new_2d(
            Geometry2DType::StarFive,
            BorderDecoration::Solid,
            Rgba::SOFT_BLACK,
            th_l,
            InnerDecoration::Solid,
            Rgba::MAGENTA,
            100,
            Vector2::new(0.0, 0.0),
            0.0,
            size,
        ),
    ));

    cmd.push((
        Transform2D::with_position(x2, x2),
        Geometry::new_2d(
            Geometry2DType::Heart,
            BorderDecoration::DynDash,
            Rgba::SOFT_BLACK,
            th_l,
            InnerDecoration::Solid,
            Rgba::ROSE,
            100,
            Vector2::new(0.0, 0.0),
            0.0,
            size,
        ),
    ));

    cmd.push((
        Transform2D::with_position(x1, x1),
        Geometry::new_1d(
            Geometry1DType::Segment,
            BorderDecoration::DynDash,
            Rgba::SOFT_BLACK,
            th_l,
            101,
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 256.0),
        ),
        Marker {},
    ));
}

#[system(for_each)]
#[filter(component::<Camera2D>())]
fn control_camera(transform: &mut Transform2D, #[resource] input: &Input) {
    const TSPEED: f32 = 4.0;
    const SSPEED: f32 = 0.40;

    if input.mouse.pressed(MouseButton::Middle) {
        let (dx, dy) = input.mouse.mouse_motion_in_ss();

        transform.position -= Vector2::<f32>::new(dx, -dy) * transform.scale.x;
    }

    let (_, motion) = input.mouse.mouse_wheel_motion();
    transform.scale = Vector2::new(
        (transform.scale.x + motion * SSPEED).max(0.2),
        (transform.scale.y + motion * SSPEED).max(0.2),
    );
}

#[system(for_each)]
#[filter(component::<Geometry>() & component::<Marker>())]
fn control_geometry(
    transform: &mut Transform2D,
    _geometry: &mut Geometry,
    #[resource] time: &Time,
) {
    const RSPEED: f32 = 60.0;

    transform.rotate(RSPEED * time.delta().as_secs_f32());
}

struct Marker;
