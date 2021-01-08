use YamEngine::app::*;
use legion::*;

fn main() {
    AppBuilder::build()
        .add_startup_system(startup_system())
        .add_layer_to_update("default", 1)
        .add_update_system(update01_system(), "default")
        .add_layer_to_update("second", 2)
        .add_update_system(update02_system(), "second")
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