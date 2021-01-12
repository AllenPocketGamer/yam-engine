use legion::*;
use yamengine::{app::*, window::*};

fn main() {
    App::build()
        .add_layer_to_update("default", 60)
        .add_update_system(mouse_input_system(), "default")
        .finish()
        .run();
}

#[system]
fn mouse_input(#[resource] input: &Input) {
    let position = input.mouse.get_position_at_window();
    let motion_delta = input.mouse.get_motion_delta();
    let wheel_delta = input.mouse.get_wheel_delta();

    let mut btns = [(MouseButton::Left, -1), (MouseButton::Right, -1), (MouseButton::Middle, -1)];

    for (btn, state) in btns.iter_mut() {
        if input.mouse.just_pressed(*btn) {
            *state = 0;

            println!("just pressed: {:?}", btn);
        }

        if input.mouse.just_released(*btn) {
            *state = 1;

            println!("just released: {:?}", btn);
        }

        if input.mouse.pressed(*btn) {
            *state = 2;
        }

        if input.mouse.released(*btn) {
            *state = 3;
        }
    }

    println!(
        "position: {:?}, motion delta: {:?}, wheel delta: {:?}; leftBtn: {}, rightBtn: {}, middleBtn: {}",
        position, motion_delta, wheel_delta, btns[0].1, btns[1].1, btns[2].1,
    );
}
