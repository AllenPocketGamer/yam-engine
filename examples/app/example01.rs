use YamEngine::app::*;
use legion::*;

fn main() {
    App::build()
        .add_startup_system(startup_system())
        .add_layer_to_update("default", 1)
        .add_update_system(update01_system(), "default")
        .add_update_system(get_app_context_system(), "default")
        .add_layer_to_update("second", 2)
        .add_update_system(update02_system(), "second")
        .add_layer_to_update("third", 3)
        .add_update_system(update03_system(), "third")
        .finish()
        .run();
}

#[system]
fn startup() {
    println!("startup system");
}

#[system]
fn update01() {
    println!("update01 system");
}

#[system]
fn update02() {
    println!("update02 system");
}

#[system]
fn update03() {
    println!("update03 system");
}

#[system]
fn get_app_context(#[resource] app_context: &AppSettings) {
    println!("call app_context.set_render_framerate");
    app_context.set_render_framerate(60);
    println!("call app_context.set_update_layer_frequency");
    app_context.set_update_layer_frequency("default", 90);
}