use input::Input;
use yamengine::app::*;
use yamengine::misc::*;
use yamengine::input::*;
use yamengine::*;

fn main() -> Result<(), AppBuildError> {
    AppBuilder::new()
        .create_stage_builder(String::from("default"))?
        .add_system_startup(parallel_startup_system())
        .add_system_process(parallel_process_system())
        .add_system_destroy(parallel_destroy_system())
        .into_app_builder()
        .build()
        .run();

    Ok(())
}

#[system]
fn parallel_startup() {
    println!("parallel startup");
}

#[system]
fn parallel_process(#[resource] input: &Input) {
    if input.mouse.just_pressed(MouseButton::Left) {
        println!("left button just pressed");
    }
}

#[system]
fn parallel_destroy() {
    println!("parallel destroy");
}