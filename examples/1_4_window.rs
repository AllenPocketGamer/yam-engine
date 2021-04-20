//! The `window` module is only in a usable state and will be gradually improved afterwawrds.

use yam::legion::*;
use yam::*;

fn main() -> Result<(), AppBuildError> {
    AppBuilder::new()
        .create_stage_builder(String::from("default"))?
        .add_thread_local_system_process(control_fullscreen_system())
        .into_app_builder()
        .build()
        .run();

    Ok(())
}

#[system]
fn control_fullscreen(#[resource] window: &mut Window, #[resource] input: &Input) {
    if input.keyboard.just_pressed(KeyCode::F) {
        if let Some(_) = window.fullscreen() {
            window.set_fullscreen(None);
        } else {
            window.set_fullscreen(Some(Fullscreen::Borderless(window.current_monitor())));
        }
    }
}
