use yam::*;
use yam::legion::*;

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
    let mbtns = [MouseButton::Left, MouseButton::Right, MouseButton::Middle];
    let kbtns = [KeyCode::A, KeyCode::S, KeyCode::D, KeyCode::W, KeyCode::Space, KeyCode::F];

    for mbtn in &mbtns {
        if input.mouse.just_pressed(*mbtn) {
            println!("{:?} just pressed", mbtn);
        }
        if input.mouse.just_released(*mbtn) {
            println!("{:?} just released", mbtn);
        }
    }

    if input.mouse.cursor_just_entered() {
        println!("cursor just entered");
    }
    if input.mouse.cursor_just_left() {
        println!("cursor just left");
    }

    for kbtn in &kbtns {
        if input.keyboard.just_pressed(*kbtn) {
            println!("{:?} just pressed", kbtn);
        }
        if input.keyboard.just_released(*kbtn) {
            println!("{:?} just released", kbtn);
        }
    }
}

#[system]
fn parallel_destroy() {
    println!("parallel destroy");
}
