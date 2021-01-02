use legion::*;

fn main() {
    let mut world = World::default();

    let _entity = world.push((Position {x: 0., y: 0.}, Velocity {dx: 0., dy: 0.}));
    let _entities = world.extend(vec![
        (Position {x: 0., y: 0.}, Velocity {dx: 0.1, dy: 0.1}),
        (Position {x: 0., y: 0.}, Velocity {dx: 0.2, dy: 0.2}),
        (Position {x: 0., y: 0.}, Velocity {dx: 0.3, dy: 0.3}),
    ]);

    // you define a query be declaring what components you want to find, and how you will access them
    let mut query = <(&Velocity, &mut Position)>::query();

    // you can then iterate through the components found in the world
    for (velocity, position) in query.iter_mut(&mut world) {
        position.x += velocity.dx;
        position.y += velocity.dy;
    }

    let mut query = <&Position>::query();
    for position in query.iter(&world) {
        println!("{:?}", position);
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