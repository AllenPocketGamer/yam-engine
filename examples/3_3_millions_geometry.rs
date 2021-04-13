use yam::legion::{systems::CommandBuffer, *};
use yam::nalgebra::Vector2;
use yam::*;

const SQRT_COUNT: usize = 1024;
const GEOM_COUNT: usize = SQRT_COUNT * SQRT_COUNT;
const QUAD_SIZE: f32 = 128.0;

fn main() -> Result<(), AppBuildError> {
    AppBuilder::new()
        .create_stage_builder(String::from("default"))?
        .add_thread_local_system_startup(introduction_system())
        .add_thread_local_system_startup(init_entities_system())
        .add_thread_local_system_process(control_camera_system())
        .add_thread_local_system_process(wander_system(0.0, 64.0, 16.0))
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
    println!("  3. Pressed A/D to control radius, S/W to control distance.");
}

#[system]
fn init_entities(commands: &mut CommandBuffer) {
    // Push camera entity to `World`.
    commands.push((Transform2D::default(), Camera2D::default()));

    // `+1` prevent double the capacity of the vec when push element into.
    let mut steerings: Instance<Steering> = Instance::with_capacity(GEOM_COUNT + 1);
    let mut transform2ds: Instance<Transform2D> = Instance::with_capacity(GEOM_COUNT + 1);

    for x in 0..SQRT_COUNT {
        for y in 0..SQRT_COUNT {
            let (tx, ty) = (QUAD_SIZE * x as f32, QUAD_SIZE * y as f32);

            steerings.push(Steering::default());
            transform2ds.push(Transform2D::with_position(tx, ty));
        }
    }

    // Push geometry(with instance) entity to `World`.
    commands.push((
        transform2ds,
        vec![
            // main geometry
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
                QUAD_SIZE,
            ),
            // // radius geometry
            // Geometry::new_2d(
            //     Geometry2DType::Circle,
            //     BorderDecoration::DynDash,
            //     Rgba::SOFT_BLACK,
            //     BorderThickness::LocalSpace(4.0),
            //     InnerDecoration::None,
            //     Rgba::WHITE,
            //     1,
            //     Vector2::new(0.0, *init_distance),
            //     0.0,
            //     2.0 * (*init_radius),
            // ),
        ],
        steerings,
    ));
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

#[system(for_each)]
#[filter(component::<Assembly>())]
fn wander(
    trf2ds: &mut Instance<Transform2D>,
    _asmbly: &mut Assembly,
    strngs: &mut Instance<Steering>,
    #[resource] input: &Input,
    #[resource] time: &Time,
    #[state] timer: &mut f32,
    #[state] p_radius: &mut f32,
    #[state] p_distance: &mut f32,
) {
    use rayon::prelude::*;

    const INTERVAL: f32 = 1.0;
    const TSPEED: f32 = 16.0;

    let delta = time.delta().as_secs_f32();

    if *timer >= INTERVAL {
        trf2ds
            .par_iter_mut()
            .zip(strngs.par_iter_mut())
            .for_each(|(trf2d, strng)| {
                let wander_force: Vector2<f32> = strng.wander(trf2d, *p_radius, *p_distance);
                strng.apply_force(&wander_force);
            });

        *timer -= INTERVAL;
    }

    trf2ds
        .par_iter_mut()
        .zip(strngs.par_iter_mut())
        .for_each(|(trf2d, strng)| {
            strng.motion(trf2d, delta);
        });

    *timer += time.delta().as_secs_f32();

    if input.keyboard.pressed(KeyCode::A) {
        *p_radius -= TSPEED * delta;
    } else if input.keyboard.pressed(KeyCode::D) {
        *p_radius += TSPEED * delta;
    }

    if input.keyboard.pressed(KeyCode::S) {
        *p_distance -= TSPEED * delta;
    } else if input.keyboard.pressed(KeyCode::W) {
        *p_distance += TSPEED * delta;
    }

    // {
    //     let r_geo = &mut asmbly[1];

    //     r_geo.set_position_uncheck(&Vector2::new(0.0, *p_distance));
    //     r_geo.set_size_uncheck(2.0 * (*p_radius));
    // }
}

#[allow(dead_code)]
struct Steering {
    velocity: Vector2<f32>,
    force: Vector2<f32>,
}

impl Steering {
    #[allow(dead_code)]
    pub const MAX_SPEED: f32 = 256.0;
    #[allow(dead_code)]
    pub const MAX_FORCE: f32 = 512.0;
    #[allow(dead_code)]
    pub const THREHOLD: f32 = 0.0001;

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

        let jitter: Vector2<f32> =
            Vector2::new(gen_random_f32(), gen_random_f32()).normalize() * r_radius;

        let to_target: Vector2<f32> = jitter + transform2d.heading_y() * r_distance;

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
            transform2d.set_heading_y(&self.velocity);
        }
    }
}

impl Default for Steering {
    fn default() -> Self {
        Self {
            velocity: Vector2::new(0.0001, 0.0001),
            force: Vector2::new(0.0, 0.0),
        }
    }
}
