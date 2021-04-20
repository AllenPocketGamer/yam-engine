//! `yam` is a game engine using ECS(Entity-Component-System) architecture.
//!
//! [What is ECS?](https://en.wikipedia.org/wiki/Entity_component_system)
//!
//! `yam` uses the `legion` which is easy to use, feature-rich high-performance
//! ECS library as the framework of engine.
//!
//! Don't worry, ECS is a simple concept and `legion` is easy to learn, you will
//! get more useful information from [there](https://github.com/amethyst/legion).

use yam::legion::*;
use yam::*;

fn main() -> Result<(), AppBuildError> {
    // Create `Appbuilder` to build a `App`.
    AppBuilder::new()
        // `AppStage` is the basic module of yam, `App` consists of serveral `AppStage`s.
        //
        // When the `App` starts running, it will execute the `AppStage`s it owns in turn.
        //
        // Create `AppStageBuilder` which used to build a `AppStage` from `AppBuilder`.
        .create_stage_builder(String::from("default"))?
        // `AppStage` has three different types of callbacks: `startup`, `process` and `destroy`.
        //
        // `startup` will be called only once at the start of the running of the `App`.
        // `process` will be called every frame at the running of the `App`.
        // `destroy` will be called only once at the end of the running of the `App`.
        //
        // Each callback has one of three states: `parallel_system`, `thread_local_system` and `thread_local_fn`.
        //
        // `parallel_system` tries to run the callback on as many threads as possible.
        // `thread_local_system` runs the callback on the main thread.
        // `thread_local_fn` runs the callback on the main thread and takes the data storage(`World` and `Resources`) as arguments.
        .add_system_startup(parallel_startup_system())
        .add_system_process(parallel_process_system())
        .add_system_destroy(parallel_destroy_system())
        .add_thread_local_system_startup(thread_local_startup_system())
        .add_thread_local_system_process(thread_local_process_system())
        .add_thread_local_system_destroy(thread_local_destroy_system())
        .add_thread_local_fn_startup(thread_local_fn_startup)
        .add_thread_local_fn_process(thread_local_fn_process)
        .add_thread_local_fn_destroy(thread_local_fn_destroy)
        // Convert to `AppBuilder` after finish building the `AppStage`.
        .into_app_builder()
        // Consume the `AppBuilder` to build the `App`.
        .build()
        // Hijack the main thread to run the `App`.
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
fn thread_local_process() {
    println!("thread local process");
}

#[system]
fn thread_local_destroy() {
    println!("thread local destroy");
}

fn thread_local_fn_startup(_world: &mut World, _resources: &mut Resources) {
    println!("thread local fn startup");
}

fn thread_local_fn_process(_world: &mut World, _resources: &mut Resources) {
    println!("thread local fn process");
}

fn thread_local_fn_destroy(_world: &mut World, _resources: &mut Resources) {
    println!("thread local fn destroy");
}
