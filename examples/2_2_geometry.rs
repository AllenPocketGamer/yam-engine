use yam::legion::{systems::CommandBuffer, *};
use yam::nalgebra::Vector2;
use yam::*;

fn main() -> Result<(), AppBuildError> {
    AppBuilder::new()
        .create_stage_builder(String::from("default"))?
        .add_thread_local_system_startup(init_entities_system())
        .add_thread_local_system_process(control_camera_system())
        // .add_thread_local_system_process(control_geometry_tmp_system())
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

    // Circle
    cmd.push((
        Transform2D::with_position(-256.0, -256.0),
        Geometry::new(
            GeometryType::Circle,
            Extra::new_cla(Vector2::new(0.0, 0.0), 0.0, 128.0),
            BorderType::Dash,
            Rgba::SOFT_BLACK,
            0.1,
            InnerType::Solid,
            Rgba::ORANGE,
            100,
        ),
    ));

    // Triangle
    cmd.push((
        Transform2D::with_position(-96.0, -256.0),
        Geometry::new(
            GeometryType::ETriangle,
            Extra::new_cla(Vector2::new(-0.0, 0.0), 0.0, 128.0),
            BorderType::Dash,
            Rgba::SOFT_BLACK,
            0.1,
            InnerType::Solid,
            Rgba::YELLOW,
            100,
        ),
    ));

    // Square
    cmd.push((
        Transform2D::with_position(64.0, -256.0),
        Geometry::new(
            GeometryType::Square,
            Extra::new_cla(Vector2::new(0.0, 0.0), 0.0, 128.0),
            BorderType::Dash,
            Rgba::SOFT_BLACK,
            0.1,
            InnerType::Solid,
            Rgba::CHARTREUSE,
            100,
        ),
        Marker {},
    ));


    // Pentagon
    cmd.push((
        Transform2D::with_position(-256.0, -96.0),
        Geometry::new(
            GeometryType::Pentagon,
            Extra::new_cla(Vector2::new(0.0, 0.0), 0.0, 128.0),
            BorderType::Dash,
            Rgba::SOFT_BLACK,
            0.1,
            InnerType::Solid,
            Rgba::SPRING,
            100,
        ),
    ));

    // Hexagon
    cmd.push((
        Transform2D::with_position(-96.0, -96.0),
        Geometry::new(
            GeometryType::Hexagon,
            Extra::new_cla(Vector2::new(0.0, 0.0), 0.0, 128.0),
            BorderType::Dash,
            Rgba::SOFT_BLACK,
            0.1,
            InnerType::Solid,
            Rgba::AZURE,
            100,
        ),
    ));

    // Octogon
    cmd.push((
        Transform2D::with_position(64.0, -96.0),
        Geometry::new(
            GeometryType::Octogon,
            Extra::new_cla(Vector2::new(0.0, 0.0), 0.0, 128.0),
            BorderType::Dash,
            Rgba::SOFT_BLACK,
            0.1,
            InnerType::Solid,
            Rgba::CYAN,
            100,
        ),
    ));

    // Hexagram
    cmd.push((
        Transform2D::with_position(-256.0, 64.0),
        Geometry::new(
            GeometryType::Hexagram,
            Extra::new_cla(Vector2::new(0.0, 0.0), 0.0, 128.0),
            BorderType::Dash,
            Rgba::SOFT_BLACK,
            0.1,
            InnerType::Solid,
            Rgba::MAGENTA,
            100,
        ),
    ));

    // StarFive
    cmd.push((
        Transform2D::with_position(-96.0, 64.0),
        Geometry::new(
            GeometryType::StarFive,
            Extra::new_cla(Vector2::new(0.0, 0.0), 0.0, 128.0),
            BorderType::Dash,
            Rgba::SOFT_BLACK,
            0.1,
            InnerType::Solid,
            Rgba::VIOLET,
            100,
        ),
    ));

    // Heart
    cmd.push((
        Transform2D::with_position(64.0, 64.0),
        Geometry::new(
            GeometryType::Heart,
            Extra::new_cla(Vector2::new(0.0, 0.0), 0.0, 128.0),
            BorderType::Dash,
            Rgba::SOFT_BLACK,
            0.1,
            InnerType::Solid,
            Rgba::ROSE,
            100,
        ),
    ));
}

#[system(for_each)]
#[filter(component::<Camera2D>())]
fn control_camera(transform: &mut Transform2D, #[resource] input: &Input) {
    const TSPEED: f32 = 4.0;
    const SSPEED: f32 = 0.40;

    if input.mouse.pressed(MouseButton::Middle) {
        let (dx, dy) = input.mouse.mouse_motion();

        transform.position -= Vector2::<f32>::new(dx, -dy) * transform.scale.x;
    }

    let (_, motion) = input.mouse.mouse_wheel_motion();
    transform.scale = Vector2::new(
        (transform.scale.x + motion * SSPEED).max(0.2),
        (transform.scale.y + motion * SSPEED).max(0.2),
    );
}

#[system(for_each)]
#[filter(component::<Geometry>() & !component::<Marker>())]
fn control_geometry_tmp(
    transform: &mut Transform2D,
    geometry: &mut Geometry,
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

    // let angle = time.delta().as_secs_f32() * 30.0;
    // transform.rotate(angle);
    // unsafe {
    //     let past = time.time().as_secs_f32();

    //     geometry.extras.cla.0.y = 32.0 * f32::sin(geometry.extras.cla.0.x + 4.0 * past);
    //     geometry.extras.cla.2 += angle;
    // }
}

struct Marker;
