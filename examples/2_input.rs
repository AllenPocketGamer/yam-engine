use yam::legion::*;
use yam::*;

fn main() -> Result<(), AppBuildError> {
    AppBuilder::new()
        .create_stage_builder(String::from("default"))?
        .add_system_process(parallel_process_system())
        .add_thread_local_system_process(thread_local_process_system())
        .into_app_builder()
        .build()
        .run();

    Ok(())
}

// Get `Input` from `Resources`.
//
// `Resources` is the data container which stores the shared datas can be retrieved in systems.
//
// `Resources` is the concept from `legion`, you will get more information from [there](https://github.com/amethyst/legion).
#[system]
fn parallel_process(#[resource] input: &Input) {
    let mbtn = MouseButton::Left;

    // The difference in the position of the mouse between two frames.
    let _mouse_motion = input.mouse.mouse_motion();

    if input.mouse.just_pressed(mbtn) {
        // The left button of mouse has just been pressed.
        println!("The left button of mouse has just been pressed.");
    } else if input.mouse.just_released(mbtn) {
        // The left button of mouse has just been released.
        println!("The right button of mouse has just been released.");
    }

    if input.mouse.pressed(mbtn) {
        // The left button of mouse has been pressed.
    } else if input.mouse.released(mbtn) {
        // The left button of mouse has been released.
    }

    if input.mouse.cursor_just_entered() {
        // The cursor has just been entered the window.
        println!("The cursor has just been entered the window.");
    } else if input.mouse.cursor_just_left() {
        // The cursor has just been left the window.
        println!("The cursor has just been left the window.");
    }

    if input.mouse.cursor_entered() {
        // The cursor has been entered the window.
    } else if input.mouse.cursor_left() {
        // The cursor has been left the window.
    }

    let kbtn = KeyCode::Space;

    if input.keyboard.just_pressed(kbtn) {
        // The space button of keyboard has just been pressed.
        println!("The space button of keyboard has just been pressed.");
    } else if input.keyboard.just_released(kbtn) {
        // The space button of keyboard has just been released.
        println!("The space button of keyboard has just been released.");
    }

    if input.keyboard.pressed(kbtn) {
        // The space button of keyboard has been pressed.
    } else if input.keyboard.released(kbtn) {
        // The space button of keyboard has been released.
    }
}

// Of course, `thread_local_system` can get the `Input` from `Resources` as well as `parallel_system`.
#[system]
fn thread_local_process(#[resource] _input: &Input) {
    // NOTE: You can do the same things as in the `parallel_system`.
}
