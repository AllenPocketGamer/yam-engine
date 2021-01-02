use legion::*;

fn main() {
    let mut world = World::default();
    let mut resources = Resources::default();

    let _entity = world.push((Position { x: 0., y: 0. }, Velocity { dx: 0., dy: 0. }, Ignore));
    let _entities = world.extend(vec![
        (Position { x: 0., y: 0. }, Velocity { dx: 0.1, dy: 0.1 }),
        (Position { x: 0., y: 0. }, Velocity { dx: 0.2, dy: 0.2 }),
        (Position { x: 0., y: 0. }, Velocity { dx: 0.3, dy: 0.3 }),
    ]);

    resources.insert::<Time>(Time {delta: 0.1});

    Schedule::builder()
        .add_system(update_positions_system())
        .add_system(print_positions_with_filter_system())
        .build()
        .execute(&mut world, &mut resources);
}

// a system fn which loops through Position and Velocity components, and reads the Time shared resource
// the #[system] macro generate a fn called update_positions_system() which will construct our system
#[system(for_each)]
fn update_positions(pos: &mut Position, vel: &Velocity, #[resource] time: &Time) {
    pos.x += vel.dx * time.delta;
    pos.y += vel.dy * time.delta;
}

#[system(for_each)]
#[filter(!component::<Ignore>() & maybe_changed::<Position>())]
fn print_positions_with_filter(pos: &Position) {
    println!("pos: {:?}", pos);
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Velocity {
    dx: f32,
    dy: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Ignore;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Time {
    delta: f32,
}