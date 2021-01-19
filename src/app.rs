use super::{misc::Timer, window::Window};
use legion::{
    systems::{Builder, ParallelRunnable, Runnable},
    Resources, Schedule, World,
};
use std::panic;

pub struct App {
    stages: Vec<AppStage>,
}

impl App {
    pub fn builder() -> AppBuilder {
        Default::default()
    }

    pub fn play(mut self) {
        todo!()
    }

    fn add_stage(&mut self, stage: AppStage) {
        todo!()
    }

    fn remove_stage(&mut self, stage_name: &str) -> AppStage {
        todo!()
    }

    fn stage(&self, stage_name: &str) -> &AppStage {
        todo!()
    }

    fn stage_mut(&mut self, stage_name: &str) -> &mut AppStage {
        todo!()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::builder().build()
    }
}

pub struct AppBuilder {
    stage_builders: Vec<AppStageBuilder>,
}

impl AppBuilder {
    pub fn new() -> Self {
        todo!()
    }

    pub fn build(self) -> App {
        todo!()
    }
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AppStage {
    name: String,
    timer: Timer,

    schedule_startup: Schedule,
    schedule_process: Schedule,
    schedule_destroy: Schedule,
}

impl AppStage {
    pub fn builder() -> AppStageBuilder {
        Default::default()
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn frequency(&self) -> u32 {
        self.timer.target_ticks
    }

    pub fn set_frequency(&mut self, frequency: u32) {
        self.timer.target_ticks = frequency;
    }

    pub(crate) fn init(&mut self, world: &mut World, resources: &mut Resources) {
        self.schedule_startup.execute(world, resources);
    }

    pub(crate) fn play(&mut self, world: &mut World, resources: &mut Resources) {
        self.timer.update();

        if self.timer.tick() {
            self.schedule_process.execute(world, resources);
        }
    }

    pub(crate) fn free(&mut self, world: &mut World, resources: &mut Resources) {
        self.schedule_destroy.execute(world, resources);
    }
}

impl Default for AppStage {
    fn default() -> Self {
        Self::builder().build()
    }
}

pub struct AppStageBuilder {
    name: String,
    frequency: u32,

    builder_startup: Builder,
    builder_process: Builder,
    builder_destroy: Builder,
}

impl AppStageBuilder {
    pub fn new(name: String, frequency: u32) -> Self {
        Self {
            name,
            frequency,

            builder_startup: Builder::default(),
            builder_process: Builder::default(),
            builder_destroy: Builder::default(),
        }
    }

    pub fn add_system_startup<T: ParallelRunnable + 'static>(&mut self, system: T) {
        self.builder_startup.add_system(system);
    }

    pub fn add_system_process<T: ParallelRunnable + 'static>(&mut self, system: T) {
        self.builder_process.add_system(system);
    }

    pub fn add_system_destroy<T: ParallelRunnable + 'static>(&mut self, system: T) {
        self.builder_destroy.add_system(system);
    }

    pub fn add_thread_local_system_startup<T: Runnable + 'static>(&mut self, system: T) {
        self.builder_startup.add_thread_local(system);
    }

    pub fn add_thread_local_system_process<T: Runnable + 'static>(&mut self, system: T) {
        self.builder_process.add_thread_local(system);
    }

    pub fn add_thread_local_system_destroy<T: Runnable + 'static>(&mut self, system: T) {
        self.builder_destroy.add_thread_local(system);
    }

    pub fn add_thread_local_fn_startup<F: FnMut(&mut World, &mut Resources) + 'static>(&mut self, f: F) {
        self.builder_startup.add_thread_local_fn(f);
    }

    pub fn add_thread_local_fn_process<F: FnMut(&mut World, &mut Resources) + 'static>(&mut self, f: F) {
        self.builder_process.add_thread_local_fn(f);
    }

    pub fn add_thread_local_fn_destroy<F: FnMut(&mut World, &mut Resources) + 'static>(&mut self, f: F) {
        self.builder_destroy.add_thread_local_fn(f);
    }

    pub fn build(mut self) -> AppStage {
        AppStage {
            name: self.name,
            timer: Timer::new(self.frequency),

            schedule_startup: self.builder_startup.build(),
            schedule_process: self.builder_process.build(),
            schedule_destroy: self.builder_destroy.build(),
        }
    }
}

impl Default for AppStageBuilder {
    fn default() -> Self {
        Self::new(String::from("default"), 60)
    }
}
