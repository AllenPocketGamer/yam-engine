use yamengine::app::*;
use yamengine::misc::*;
use yamengine::*;

fn main() -> Result<(), AppBuildError> {
    AppBuilder::new()
        .create_stage_builder(String::from("default"), 60)?
        .add_system_startup(parallel_startup_system())
        .add_system_process(parallel_process_system())
        .add_system_destroy(parallel_destroy_system())
        .add_thread_local_system_startup(thread_local_startup_system())
        .add_thread_local_system_process(thread_local_process_system())
        .add_thread_local_system_destroy(thread_local_destroy_system())
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
fn parallel_process() {
    println!("parallel process");
}

#[system]
fn parallel_destroy() {
    println!("parallel destroy");
}

#[system]
fn thread_local_startup() {
    println!("thread local startup");
}

#[system]
/// `AppSettings` is thread-local resource 
fn thread_local_process(#[resource] timer: &PulseTimer, #[resource] settings: &mut AppSettings) {
    if timer.total_time().as_secs_f32() > 2.0 {
        settings.quit();

        println!("exceed 2 seconds, exit");
    } else {
        println!("total_time: {}; delta: {}", timer.total_time().as_secs_f32(), timer.delta().as_secs_f32());
    }
}

#[system]
fn thread_local_destroy() {
    println!("thread local destroy");
}
