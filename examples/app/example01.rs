use YamEngine::app::*;
use legion::*;

fn main() {
    App::build()
        .add_startup_system(startup_system())
        .add_layer_to_update("default", 1)
        .add_update_system(get_app_context_system(), "default")
        .finish()
        .run();
}

#[system]
fn startup() {
    println!("startup system");
}

#[system]
fn get_app_context(#[resource] timer: &Timer, #[resource] app_context: &mut AppSettings) {
    println!("time: {:?}", timer);
    app_context.set_update_layer_frequency("default", 10);
}