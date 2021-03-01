use yam::*;
use yam::legion::*;
use yam::nalgebra::Vector2;

fn main() -> Result<(), AppBuildError> {
    AppBuilder::new()
        .create_stage_builder(String::from("default"))?
        .add_thread_local_fn_startup(init_entities)
        .add_thread_local_system_process(operate_camera_system())
        .add_thread_local_system_process(steering_sprites_system(ProfileTimer::new()))
        .into_app_builder()
        .build()
        .run();

    Ok(())
}

fn init_entities(world: &mut World, _resources: &mut Resources) {
    const SIZE: f32 = 8.0;
    const SQRT: usize = 1000;

    world.push((Transform2D::default(), Camera2D::new(1920, 1080)));

    let mut transform2ds = Vec::<Transform2D>::with_capacity(SQRT * SQRT + 8);
    let mut steerings = Vec::<Steering>::with_capacity(SQRT * SQRT + 8);

    for x in 0..SQRT {
        for y in 0..SQRT {
            let (tx, ty) = (1.2 * SIZE * x as f32, 1.2 * SIZE * y as f32);

            transform2ds.push(Transform2D::new(tx, ty, 0.0, SIZE, SIZE));
            steerings.push(Steering::new(0.0, 0.0));
        }
    }

    world.push((transform2ds, steerings, Sprite { color: Color::BLUE }));
}

#[system(for_each)]
#[filter(component::<Camera2D>())]
fn operate_camera(transform: &mut Transform2D, #[resource] input: &Input) {
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
#[filter(component::<Sprite>())]
fn steering_sprites(
    #[state] ptimer: &mut ProfileTimer,
    transform2ds: &mut Vec<Transform2D>,
    steerings: &mut Vec<Steering>,
    #[resource] time: &Time,
) {
    use rayon::prelude::*;

    const RADIUS: f32 = 8.0;
    const DISTANCE: f32 = 4.0;

    let delta = time.delta().as_secs_f32();

    ptimer.start_record();

    transform2ds
        .par_iter_mut()
        .zip(steerings.par_iter_mut())
        .for_each(|(transform2d, steering)| {
            steering.apply_force(&steering.wander(transform2d, RADIUS, DISTANCE));
            steering.motion(transform2d, delta);
        });

    ptimer.stop_record();

    let tmp = *ptimer;
    std::thread::spawn(move || {
        println!("{}", tmp);
    });
}

#[allow(dead_code)]
struct Steering {
    velocity: Vector2<f32>,
    force: Vector2<f32>,
}

impl Steering {
    #[allow(dead_code)]
    pub const MAX_SPEED: f32 = 128.0;
    #[allow(dead_code)]
    pub const MAX_FORCE: f32 = 256.0;
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