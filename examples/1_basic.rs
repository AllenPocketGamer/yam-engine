use yam::*;
use yam::legion::*;

fn main() -> Result<(), AppBuildError> {
    AppBuilder::new()
        .create_stage_builder(String::from("default"))?
        .add_system_startup(parallel_startup_system())
        .add_system_process(parallel_process_system())
        .add_system_destroy(parallel_destroy_system())
        .add_thread_local_system_startup(thread_local_startup_system())
        .add_thread_local_system_process(thread_local_process_system(0f32))
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
fn thread_local_process(#[state] sum_freq: &mut f32, #[resource] time: &Time, #[resource] settings: &mut AppSettings) {
    if time.time().as_secs_f32() > 2.0 {
        settings.quit();

        println!("exceed 2 seconds, exit");
    } else {
        *sum_freq += 1.0 / time.delta().as_secs_f32();
        let average_freq = *sum_freq / time.record_count() as f32;

        println!("time: {}, delta: {}, average freq: {}", time.time().as_secs_f32(), time.delta().as_secs_f32(), average_freq);
    }
}

#[system]
fn thread_local_destroy() {
    println!("thread local destroy");
}
