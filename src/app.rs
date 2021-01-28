use super::{misc::Timer, window::Window};
use legion::{
    systems::{Builder, ParallelRunnable, Runnable},
    Resources, Schedule, World,
};
use std::{
    cell::RefCell,
    fmt, panic,
    rc::Rc,
    slice::{Iter, IterMut},
};

#[derive(Default)]
pub struct App {
    busy_stages: Vec<AppStage>,
    spare_stages: Vec<AppStage>,
}

impl App {
    pub fn new() -> Self {
        Self {
            busy_stages: Default::default(),
            spare_stages: Default::default(),
        }
    }

    pub fn from_stages(stages: Vec<AppStage>) -> Self {
        Self {
            busy_stages: stages,
            spare_stages: Default::default(),
        }
    }

    pub fn run(self) {
        // share ownership and provide iterior mutability
        let app = Rc::new(RefCell::new(self));
        let commands = Rc::new(RefCell::new(Vec::<AppCommand>::with_capacity(4)));

        fn apply_settings(resources: &mut Resources, app: &Rc<RefCell<App>>, commands: &Rc<RefCell<Vec<AppCommand>>>) -> bool {
            if resources.contains::<AppSettings>() {
                resources.get::<AppSettings>().unwrap().apply()
            } else {
                commands.borrow_mut().clear();
                resources.insert::<AppSettings>(AppSettings::new(app, commands));
                false
            }
        }

        let mut world = World::default();
        let mut resources = Resources::default();

        resources.insert::<AppSettings>(AppSettings::new(&app, &commands));

        for stage in RefCell::borrow(&app).busy_stages.iter() {
            stage.init(&mut world, &mut resources);
        }

        while !apply_settings(&mut resources, &app, &commands) {
            for stage in RefCell::borrow(&app).busy_stages.iter() {
                stage.play(&mut world, &mut resources);
            }
        }

        for stage in RefCell::borrow(&app).busy_stages.iter() {
            stage.free(&mut world, &mut resources);
        }
    }
}

#[derive(Default)]
pub struct AppBuilder {
    stage_builders: Vec<AppStageBuilder>,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            stage_builders: Default::default(),
        }
    }

    pub fn add_stage_builder(mut self, stage_builder: AppStageBuilder) -> Result<Self, AppBuildError> {
        if self.has_stage(stage_builder.name()) {
            Err(AppBuildError::DuplicateName(stage_builder))
        } else {
            self.stage_builders.push(stage_builder);
            Ok(self)
        }
    }

    pub fn create_stage_builder(self, stage_name: String, frequency: u32) -> Result<AppStageBuilder, AppBuildError> {
        let mut stage_builder = AppStageBuilder::new(stage_name, frequency);

        if self.has_stage(stage_builder.name()) {
            Err(AppBuildError::DuplicateName(stage_builder))
        } else {
            stage_builder.app_builder.replace(self);
            Ok(stage_builder)
        }
    }

    pub fn build(self) -> App {
        App::from_stages(self.stage_builders.into_iter().map(|stage_builder| stage_builder.build()).collect())
    }

    fn has_stage(&self, stage_name: &str) -> bool {
        self.stage_builders.iter().find(|stage| stage.name() == stage_name).is_some()
    }
}

pub struct AppStage {
    name: String,
    timer: RefCell<Timer>,

    startup: RefCell<Schedule>,
    process: RefCell<Schedule>,
    destroy: RefCell<Schedule>,
}

impl AppStage {
    fn new(name: String, timer: Timer, startup: Schedule, process: Schedule, destroy: Schedule) -> Self {
        Self {
            name,
            timer: RefCell::new(timer),

            startup: RefCell::new(startup),
            process: RefCell::new(process),
            destroy: RefCell::new(destroy),
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn frequency(&self) -> u32 {
        self.timer.borrow().target_ticks
    }

    pub fn set_frequency(&mut self, frequency: u32) {
        self.timer.borrow_mut().target_ticks = frequency;
    }

    pub(crate) fn init(&self, world: &mut World, resources: &mut Resources) {
        self.startup.borrow_mut().execute(world, resources);
    }

    pub(crate) fn play(&self, world: &mut World, resources: &mut Resources) {
        self.timer.borrow_mut().update();

        if self.timer.borrow_mut().tick() {
            resources.insert::<Timer>(*self.timer.borrow());

            self.process.borrow_mut().execute(world, resources);
        }
    }

    pub(crate) fn free(&self, world: &mut World, resources: &mut Resources) {
        self.destroy.borrow_mut().execute(world, resources);
    }
}

pub struct AppStageBuilder {
    name: String,
    frequency: u32,

    builder_startup: Builder,
    builder_process: Builder,
    builder_destroy: Builder,

    app_builder: Option<AppBuilder>,
}

impl AppStageBuilder {
    pub fn new(name: String, frequency: u32) -> Self {
        Self {
            name,
            frequency,

            builder_startup: Builder::default(),
            builder_process: Builder::default(),
            builder_destroy: Builder::default(),

            app_builder: None,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn frequency(&self) -> u32 {
        self.frequency
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
        AppStage::new(
            self.name,
            Timer::new(self.frequency),
            self.builder_startup.build(),
            self.builder_process.build(),
            self.builder_destroy.build(),
        )
    }

    pub fn into_app_builder(mut self) -> AppBuilder {
        let app_builder = if self.app_builder.is_some() {
            self.app_builder.take().unwrap()
        } else {
            AppBuilder::default()
        };

        app_builder.add_stage_builder(self).ok().unwrap()
    }
}

pub struct AppSettings {
    app: Rc<RefCell<App>>,
    commands: Rc<RefCell<Vec<AppCommand>>>,
}

impl AppSettings {
    fn new(app: &Rc<RefCell<App>>, commands: &Rc<RefCell<Vec<AppCommand>>>) -> Self {
        Self {
            app: Rc::clone(app),
            commands: Rc::clone(commands),
        }
    }

    fn apply(&self) -> bool {
        todo!()
    }

    pub fn busy_stage<'a>(&'a self, stage_name: &str) -> Option<&'a AppStage> {
        let stages: &'a Vec<AppStage> = unsafe {
            // TODO: write safety words
            &self.app.try_borrow_unguarded().unwrap().busy_stages
        };

        stages.iter().find(|stage| stage.name() == stage_name)
    }

    pub fn busy_stage_iter<'a>(&'a self) -> Iter<'a, AppStage> {
        let stages: &'a Vec<AppStage> = unsafe {
            // TODO: write safety words
            &self.app.try_borrow_unguarded().unwrap().busy_stages
        };

        stages.iter()
    }

    pub fn spare_stage<'a>(&'a self, stage_name: &str) -> Option<&'a AppStage> {
        let stages: &'a Vec<AppStage> = unsafe {
            // TODO: write safety words
            &self.app.try_borrow_unguarded().unwrap().spare_stages
        };

        stages.iter().find(|stage| stage.name() == stage_name)
    }

    pub fn spare_stage_iter<'a>(&'a self) -> Iter<'a, AppStage> {
        let stages: &'a Vec<AppStage> = unsafe {
            // TODO: write safety words
            &self.app.try_borrow_unguarded().unwrap().spare_stages
        };

        stages.iter()
    }

    pub fn spare_stage_mut<'a>(&'a mut self, stage_name: &str) -> Option<&'a mut AppStage> {
        let stages: &'a mut Vec<AppStage> = unsafe {
            // TODO: write safety words
            &mut (*self.app.as_ptr()).spare_stages
        };

        stages.iter_mut().find(|stage| stage.name() == stage_name)
    }

    pub fn spare_stage_iter_mut<'a>(&'a mut self) -> IterMut<'a, AppStage> {
        let stages: &'a mut Vec<AppStage> = unsafe {
            // TODO: write safety words
            &mut (*self.app.as_ptr()).spare_stages
        };

        stages.iter_mut()
    }

    pub fn take_spare_stage(&mut self, stage_name: &str) -> Option<AppStage> {
        let stages = &mut self.app.borrow_mut().spare_stages;

        if let Some(index) = stages
            .iter_mut()
            .enumerate()
            .find(|(_, stage)| stage.name() == stage_name)
            .map(|(i, _)| i)
        {
            Some(stages.remove(index))
        } else {
            None
        }
    }

    pub fn is_stage_busy(&self, stage_name: &str) -> bool {
        self.app
            .borrow()
            .busy_stages
            .iter()
            .find(|stage| stage.name() == stage_name)
            .is_some()
    }

    pub fn is_stage_spare(&self, stage_name: &str) -> bool {
        todo!()
    }

    pub fn push_stage_to_work_before(&mut self, stage: AppStage, after_stage_name: &str) -> Result<(), AppRunError> {
        todo!()
    }

    pub fn push_stage_to_work(&mut self, stage: AppStage) -> Result<(), AppRunError> {
        todo!()
    }

    pub fn push_stage_to_work_after(&mut self, stage: AppStage, before_stage_name: &str) -> Result<(), AppRunError> {
        todo!()
    }

    pub fn make_spare_stage_work_before(&mut self, stage_name: &str, after_stage_name: &str) -> Result<(), AppRunError> {
        todo!()
    }

    pub fn make_spare_stage_work(&mut self, stage_name: &str) -> Result<(), AppRunError> {
        todo!()
    }

    pub fn make_spare_stage_work_after(&mut self, stage_name: &str, before_stage_name: &str) -> Result<(), AppRunError> {
        todo!()
    }

    pub fn push_stage_to_rest(&mut self, stage: AppStage) -> Result<(), AppRunError> {
        todo!()
    }

    pub fn make_busy_stage_rest(&mut self, stage_name: &str) -> Result<(), AppRunError> {
        todo!()
    }

    pub fn set_stage_frequency<'a>(&self, stage_name: &'a str, frequency: u32) -> Result<(), AppRunError<'a>> {
        if self.is_stage_spare(stage_name) {
        } else if self.is_stage_spare(stage_name) {
        }

        todo!()
    }

    pub fn quit(&self) {
        self.commands.borrow_mut().push(AppCommand::AppQuit);
    }
}

pub enum AppCommand {
    PushStageToWorkBefore { stage: AppStage, after_stage_name: String },
    PushStageToWork { stage: AppStage },
    PushStageToWorkAfter { stage: AppStage, before_stage_name: String },
    MakeBusyStageToRest { stage_name: String },
    SetBusyStageFrequency { stage_name: String, frequency: u32 },
    AppQuit,
}

pub enum AppRunError<'a> {
    DuplicateNameInBusy(AppStage),
    DuplicateNameInSpare(AppStage),
    StageNotExist(&'a str),
    StageNotExistInBusy(&'a str),
    StageNotExistInSpare(&'a str),
}

// TODO:
impl<'a> fmt::Debug for AppRunError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("").finish()
    }
}

pub enum AppBuildError {
    DuplicateName(AppStageBuilder),
}

impl fmt::Debug for AppBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("").finish()
    }
}
