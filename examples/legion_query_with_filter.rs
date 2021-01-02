use legion::*;

fn main() {
    let mut world = World::default();

    let _entity = world.push((Position { x: 0., y: 0. }, Velocity { dx: 0., dy: 0. }, Ignore));
    let _entities = world.extend(vec![
        (Position { x: 0., y: 0. }, Velocity { dx: 0.1, dy: 0.1 }),
        (Position { x: 0., y: 0. }, Velocity { dx: 0.2, dy: 0.2 }),
        (Position { x: 0., y: 0. }, Velocity { dx: 0.3, dy: 0.3 }),
    ]);

    let mut query = <(&Velocity, &mut Position)>::query();
    for (velocity, position) in query.iter_mut(&mut world) {
        position.x += velocity.dx;
        position.y += velocity.dy;
    }
    
    let mut query = <(&Velocity, &mut Position)>::query()
        .filter(!component::<Ignore>() & maybe_changed::<Position>());

    for (velocity, position) in query.iter_mut(&mut world) {
        println!("vel: {:?}, pos: {:?}", velocity, position);
    }
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
