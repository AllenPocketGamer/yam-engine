use legion::*;

fn main() {
    let mut world = World::default();

    let entity = world.push((Position {x: 0., y: 0.}, Velocity {dx: 0., dy: 0.}));
    let entities = world.extend(vec![
        (Position {x: 0., y: 0.}, Velocity {dx: 0., dy: 0.}),
        (Position {x: 0., y: 0.}, Velocity {dx: 0., dy: 0.}),
        (Position {x: 0., y: 0.}, Velocity {dx: 0., dy: 0.}),
    ]);

    println!("entity: {:?}", entity);
    println!("entities: {:?}", entities);
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