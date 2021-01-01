use legion::*;

fn main() {
    let mut world = World::default();

    let entity = world.push((Position {x: 0., y: 0.}, Velocity {dx: 0., dy: 0.}));

    // NOTE: Entry: Provides safe read and write access to an entity's components, and the ability to modify the entity.
    if let Some(mut entry) = world.entry(entity) {
        // add an extra component
        entry.add_component(12f32);

        // access information about the entity's archetype
        // NOTE: An archtype is a collection of entities which all have identical component types.
        println!("{:?} has {:?}", entity, entry.archetype().layout().component_types());

        // access the entity's components, return `None` if the entity does not have the component
        let pos = entry.get_component::<Position>().unwrap();
        let vel = entry.get_component::<Velocity>().unwrap();
        let num = entry.get_component::<f32>().unwrap();
        assert_eq!(pos, &Position {x: 0., y:0.});
        assert_eq!(vel, &Velocity {dx: 0., dy: 0.});
        assert_eq!(num, &12f32);
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