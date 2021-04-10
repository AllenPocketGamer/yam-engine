//! Run the example with command `cargo run --example render2d_millions` --release`.
//!
//! My device(CPU: AMD R5 2600X, Meomory: 16GB DDR4, GPU: NVIDIA GTX1070Ti) can be
//! able to render 1_000_000 sprites in 60 fps.

use yam::legion::{systems::CommandBuffer, *};
use yam::nalgebra::Vector2;
use yam::*;

fn main() -> Result<(), AppBuildError> {
    AppBuilder::new()
        .create_stage_builder(String::from("default"))?
        .add_thread_local_system_startup(introduction_system())
        .add_thread_local_system_startup(init_entities_system())
        .add_thread_local_system_process(control_camera_system())
        .add_thread_local_system_process(wander_system())
        .into_app_builder()
        .build()
        .run();

    Ok(())
}

#[system]
fn introduction() {
    println!("Introduction:");
    println!("  1. Pressed the middle button of mouse to move the camera.");
    println!("  2. Scroll the wheel of mouse to scale the view of the camera.");
}

#[system]
fn init_entities(commands: &mut CommandBuffer, #[resource] window: &Window) {
    const GEOMETRY_SIZE: f32 = 128.0;

    const SQRT_COUNT: usize = 512;
    const COUNT: usize = SQRT_COUNT * SQRT_COUNT;

    let (width, height) = window.resolution();

    // Push camera entity to `World`.
    commands.push((Transform2D::default(), Camera2D::new(width, height)));

    // `+8` prevent double the capacity of the vec when push element into.
    let mut steerings: Instance<Steering> = Instance::with_capacity(COUNT + 8);
    let mut transform2ds: Instance<Transform2D> = Instance::with_capacity(COUNT + 8);

    for x in 0..SQRT_COUNT {
        for y in 0..SQRT_COUNT {
            let (tx, ty) = (
                1.2 * GEOMETRY_SIZE * x as f32,
                1.2 * GEOMETRY_SIZE * y as f32,
            );

            steerings.push(Steering::new(0.0, 0.0));
            transform2ds.push(Transform2D::with_position(tx, ty));
        }
    }

    // Push geometry(with instance) entity to `World`.
    commands.push((
        transform2ds,
        Geometry2D::new(
            Geometry2DType::StarFive,
            BorderDecoration::DynDash,
            Rgba::SOFT_BLACK,
            -2.0,
            InnerDecoration::Solid,
            Rgba::VIOLET,
            1,
            Vector2::new(0.0, 0.0),
            0.0,
            GEOMETRY_SIZE,
        ),
        steerings,
    ));

    commands.push((
        Transform2D::default(),
        Geometry2D::new(
            Geometry2DType::Circle,
            BorderDecoration::DynDash,
            Rgba::SOFT_BLACK,
            -2.0,
            InnerDecoration::Solid,
            Rgba::ROSE,
            1,
            Vector2::new(0.0, 0.0),
            0.0,
            128.0,
        ),
    ));
}

#[system(for_each)]
#[filter(component::<Camera2D>())]
fn control_camera(transform: &mut Transform2D, #[resource] input: &Input) {
    const TSPEED: f32 = 16.0;
    const SSPEED: f32 = 0.40;

    if input.mouse.pressed(MouseButton::Middle) {
        let (dx, dy) = input.mouse.mouse_motion();

        transform.position += Vector2::<f32>::new(dx, -dy) * TSPEED;
    }

    let (_, motion) = input.mouse.mouse_wheel_motion();
    transform.scale = Vector2::new(
        (transform.scale.x + motion).max(0.2),
        (transform.scale.y + motion).max(0.2),
    );
}

#[system(for_each)]
#[filter(component::<Geometry2D>())]
fn wander(
    transform2ds: &mut Instance<Transform2D>,
    steerings: &mut Instance<Steering>,
    #[resource] time: &Time,
) {
    use rayon::prelude::*;

    const RADIUS: f32 = 128.0;
    const DISTANCE: f32 = 64.0;

    let delta = time.delta().as_secs_f32();

    transform2ds
        .par_iter_mut()
        .zip(steerings.par_iter_mut())
        .for_each(|(transform2d, steering)| {
            steering.apply_force(&steering.wander(transform2d, RADIUS, DISTANCE));
            steering.motion(transform2d, delta);
        });
}

#[allow(dead_code)]
struct Steering {
    velocity: Vector2<f32>,
    force: Vector2<f32>,
}

impl Steering {
    #[allow(dead_code)]
    pub const MAX_SPEED: f32 = 1024.0;
    #[allow(dead_code)]
    pub const MAX_FORCE: f32 = 2048.0;
    #[allow(dead_code)]
    pub const THREHOLD: f32 = 0.0001;

    pub fn new(x: f32, y: f32) -> Self {
        Self {
            velocity: Vector2::new(x, y),
            force: Vector2::new(0.0, 0.0),
        }
    }

    #[allow(dead_code)]
    pub fn seek(&self, transform2d: &Transform2D, target: &Vector2<f32>) -> Vector2<f32> {
        let to_target: Vector2<f32> = *target - transform2d.position;
        let desired_velocity: Vector2<f32> = to_target.normalize() * Self::MAX_FORCE;

        desired_velocity - self.velocity
    }

    pub fn wander(
        &self,
        transform2d: &Transform2D,
        r_radius: f32,
        r_distance: f32,
    ) -> Vector2<f32> {
        // from -1.0 to 1.0
        fn gen_random_f32() -> f32 {
            2.0 * (rand::random::<f32>() - 0.5)
        }

        let jitter: Vector2<f32> = Vector2::new(gen_random_f32(), gen_random_f32()) * r_radius;

        let to_target: Vector2<f32> = jitter + transform2d.heading() * r_distance;
        let desired_velocity: Vector2<f32> = to_target.normalize() * Self::MAX_FORCE;

        desired_velocity - self.velocity
    }

    pub fn apply_force(&mut self, force: &Vector2<f32>) {
        self.force = force.normalize() * Self::MAX_FORCE.min(force.norm());
    }

    pub fn motion(&mut self, transform2d: &mut Transform2D, delta: f32) {
        self.velocity += self.force * delta;
        self.velocity = self.velocity.normalize() * Self::MAX_SPEED.min(self.velocity.norm());

        transform2d.position += self.velocity * delta;

        if self.velocity.norm() > Self::THREHOLD {
            transform2d.set_heading(&self.velocity.normalize());
        }
    }
}
