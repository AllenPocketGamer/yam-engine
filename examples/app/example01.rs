use yamengine::app::*;
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
fn startup(#[resource] settings: &AppSettings) {
    for l2f in settings.iter_update_layer_to_frequency() {
        println!("{:?}", l2f);
    }
}

#[system]
fn get_app_context(#[resource] timer: &Timer, #[resource] settings: &mut AppSettings) {
    println!("{}", timer);
    settings.set_update_layer_frequency("default", 10);
}