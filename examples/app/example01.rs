use YamEngine::app::*;

struct Position {x: f32, y: f32}
struct Velocity {dx: f32, dy: f32}
struct Time {dt: f32}

#[system]
fn start() {
    println!("system only execute once on Stage::START");
}

#[system]
fn update() {
    println!("system execute looply on Stage::UPDATE");
}

#[system(for_each)]
fn entities_iter(pos: &mut Position, vel: &Velocity, #[resource] time: &Time) {
    pos.x += vel.dx * time.dt;
    pos.y += vel.dy * time.dt;
}

fn main() {
     let mut app = App::default();

     let _entity = app.add_entity((Position {x: 0.0, y: 0.0}, Velocity {dx: 0.0, dy: 0.0}));
     let _entities = app.add_entities(vec![
         (Position {x: 0.0, y: 0.0}, Velocity {dx: 0.1, dy: 0.1}),
         (Position {x: 0.0, y: 0.0}, Velocity {dx: 0.2, dy: 0.2}),
         (Position {x: 0.0, y: 0.0}, Velocity {dx: 0.3, dy: 0.3}),
     ]);

     app.add_resource(Time {dt: 0.1});

     app.add_system(Stage::START, start_system());
     app.add_system(Stage::UPDATE, update_system());

     app.add_system(Stage::UPDATE, entities_iter_system());

     app.run();
 }