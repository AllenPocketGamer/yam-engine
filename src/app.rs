use super::{misc::Timer, window::Window};
use legion::{
    systems::{Builder, ParallelRunnable, Runnable},
    Resources, Schedule, World,
};
use std::{
    fmt, panic,
    slice::{Iter, IterMut},
};

pub struct App {
    stages: Vec<AppStage>,

    world: World,
    resources: Resources,
}

// TODO: 插入stage时运行初始化
// TODO: 删除stage时释放资源
impl App {
    pub fn builder() -> AppBuilder {
        Default::default()
    }

    pub fn run(mut self) {
        for stage in self.stages.iter_mut() {
            stage.init(&mut self.world, &mut self.resources);
        }

        // TODO: 增加退出机制
        // loop {
        //     for stage in self.stages.iter_mut() {
        //         stage.play(&mut self.world, &mut self.resources);
        //     }
        // }

        for stage in self.stages.iter_mut() {
            stage.free(&mut self.world, &mut self.resources);
        }
    }

    pub(crate) fn push_stage(&mut self, stage: AppStage) {
        todo!()
    }

    pub(crate) fn insert_stage_after(&mut self, stage: AppStage, stage_name_before: &str) {
        todo!()
    }

    pub(crate) fn insert_stage_before(&mut self, stage: AppStage, stage_name_after: &str) {
        todo!()
    }

    pub(crate) fn remove_stage(&mut self, stage_name: &str) -> AppStage {
        todo!()
    }

    pub(crate) fn stage(&self, stage_name: &str) -> &AppStage {
        todo!()
    }

    pub(crate) fn stage_mut(&mut self, stage_name: &str) -> &mut AppStage {
        todo!()
    }

    pub(crate) fn stages(&self) -> std::slice::Iter<AppStage> {
        self.stages.iter()
    }

    pub(crate) fn stages_mut(&mut self) -> std::slice::IterMut<AppStage> {
        self.stages.iter_mut()
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
        App {
            stages: self.stage_builders.into_iter().map(|stage_builder| stage_builder.build()).collect(),

            world: World::default(),
            resources: Resources::default(),
        }
    }

    fn has_stage(&self, stage_name: &str) -> bool {
        self.stage_builders.iter().find(|stage| stage.name() == stage_name).is_some()
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
        AppStage {
            name: self.name,
            timer: Timer::new(self.frequency),

            schedule_startup: self.builder_startup.build(),
            schedule_process: self.builder_process.build(),
            schedule_destroy: self.builder_destroy.build(),
        }
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

impl Default for AppStageBuilder {
    fn default() -> Self {
        Self::new(String::from("default"), 60)
    }
}
pub struct AppSettings<'a> {
    busy_stages: &'a Vec<AppStage>,
    spare_stages: Vec<AppStage>,

    commands: Vec<AppCommand<'a>>,
}

impl<'a> AppSettings<'a> {
    pub(crate) fn new(app: &App) -> Self {
        todo!()
    }

    pub fn busy_stage(&self, stage_name: &str) -> Option<&AppStage> {
        self.busy_stages.iter().find(|stage| stage.name() == stage_name)
    }

    pub fn busy_stage_iter(&self) -> Iter<AppStage> {
        self.busy_stages.iter()
    }

    pub fn spare_stage(&self, stage_name: &str) -> Option<&AppStage> {
        self.spare_stages.iter().find(|stage| stage.name() == stage_name)
    }

    pub fn spare_stage_mut(&mut self, stage_name: &str) -> Option<&mut AppStage> {
        self.spare_stages.iter_mut().find(|stage| stage.name() == stage_name)
    }

    pub fn take_spare_stage(&mut self, stage_name: &str) -> Option<AppStage> {
        todo!()
    }

    pub fn spare_stage_iter(&self) -> Iter<AppStage> {
        self.spare_stages.iter()
    }

    pub fn spare_stage_iter_mut(&mut self) -> IterMut<AppStage> {
        self.spare_stages.iter_mut()
    }

    pub fn push_stage_to_work_before(&mut self, stage: AppStage, after_stage_name: &'a str) -> Result<(), AppRunError> {
        if self.is_stage_busy(after_stage_name) {
            if self.is_stage_busy(stage.name()) {
                Err(AppRunError::DuplicateNameInBusy(stage))
            } else if self.is_stage_spare(stage.name()) {
                Err(AppRunError::DuplicateNameInSpare(stage))
            } else {
                self.commands.push(AppCommand::PushStageToWorkBefore { stage, after_stage_name });
                Ok(())
            }
        } else {
            Err(AppRunError::StageNotExistInBusy(after_stage_name))
        }
    }

    pub fn push_stage_to_work(&mut self, stage: AppStage) -> Result<(), AppRunError> {
        if self.is_stage_busy(stage.name()) {
            Err(AppRunError::DuplicateNameInBusy(stage))
        } else if self.is_stage_spare(stage.name()) {
            Err(AppRunError::DuplicateNameInSpare(stage))
        } else {
            self.commands.push(AppCommand::PushStageToWork { stage });
            Ok(())
        }
    }

    pub fn push_stage_to_work_after(&mut self, stage: AppStage, before_stage_name: &'a str) -> Result<(), AppRunError> {
        if self.is_stage_busy(before_stage_name) {
            if self.is_stage_busy(stage.name()) {
                Err(AppRunError::DuplicateNameInBusy(stage))
            } else if self.is_stage_spare(stage.name()) {
                Err(AppRunError::DuplicateNameInSpare(stage))
            } else {
                self.commands.push(AppCommand::PushStageToWorkAfter { stage, before_stage_name });
                Ok(())
            }
        } else {
            Err(AppRunError::StageNotExistInBusy(before_stage_name))
        }
    }

    pub fn make_spare_stage_work_before(&mut self, stage_name: &'a str, after_stage_name: &'a str) -> Result<(), AppRunError> {
        if self.is_stage_busy(after_stage_name) {
            if self.is_stage_spare(stage_name) {
                let spare_stage = self.take_spare_stage(stage_name).unwrap();
                self.push_stage_to_work_before(spare_stage, after_stage_name)
            } else {
                Err(AppRunError::StageNotExistInSpare(stage_name))
            }
        } else {
            Err(AppRunError::StageNotExistInBusy(after_stage_name))
        }
    }

    pub fn make_spare_stage_work(&mut self, stage_name: &'a str) -> Result<(), AppRunError> {
        if self.is_stage_spare(stage_name) {
            let spare_stage = self.take_spare_stage(stage_name).unwrap();
            self.push_stage_to_work(spare_stage)
        } else {
            Err(AppRunError::StageNotExistInSpare(stage_name))
        }
    }

    pub fn make_spare_stage_work_after(&mut self, stage_name: &'a str, before_stage_name: &'a str) -> Result<(), AppRunError> {
        if self.is_stage_busy(before_stage_name) {
            if self.is_stage_spare(stage_name) {
                let spare_stage = self.take_spare_stage(stage_name).unwrap();
                self.push_stage_to_work_after(spare_stage, before_stage_name)
            } else {
                Err(AppRunError::StageNotExistInSpare(stage_name))
            }
        } else {
            Err(AppRunError::StageNotExistInBusy(before_stage_name))
        }
    }

    pub fn push_stage_to_rest(&mut self, stage: AppStage) -> Result<(), AppRunError> {
        if self.is_stage_busy(stage.name()) {
            Err(AppRunError::DuplicateNameInBusy(stage))
        } else if self.is_stage_spare(stage.name()) {
            Err(AppRunError::DuplicateNameInSpare(stage))
        } else {
            self.spare_stages.push(stage);
            Ok(())
        }
    }

    pub fn make_busy_stage_rest(&mut self, stage_name: &'a str) -> Result<(), AppRunError> {
        if self.is_stage_busy(stage_name) {
            self.commands.push(AppCommand::MakeBusyStageToRest { stage_name });
            Ok(())
        } else if self.is_stage_spare(stage_name){
            Err(AppRunError::StageNotExistInBusy(stage_name))
        } else {
            Err(AppRunError::StageNotExist(stage_name))
        }
    }

    pub fn set_stage_frequency(&mut self, stage_name: &'a str, frequency: u32) -> Result<(), AppRunError> {
        if self.is_stage_busy(stage_name) {
            self.commands.push(AppCommand::SetBusyStageFrequency {stage_name, frequency});
            Ok(())
        } else if let Some(stage) = self.spare_stage_mut(stage_name) {
            stage.set_frequency(frequency);
            Ok(())
        } else {
            Err(AppRunError::StageNotExist(stage_name))
        }
    }

    pub fn is_stage_busy(&self, stage_name: &str) -> bool {
        self.busy_stage(stage_name).is_some()
    }

    pub fn is_stage_spare(&self, stage_name: &str) -> bool {
        self.spare_stage(stage_name).is_some()
    }
}

pub enum AppCommand<'a> {
    PushStageToWorkBefore { stage: AppStage, after_stage_name: &'a str },
    PushStageToWork { stage: AppStage },
    PushStageToWorkAfter { stage: AppStage, before_stage_name: &'a str },
    MakeBusyStageToRest { stage_name: &'a str },
    SetBusyStageFrequency { stage_name: &'a str, frequency: u32 },
}

pub enum AppRunError<'a> {
    DuplicateNameInBusy(AppStage),
    DuplicateNameInSpare(AppStage),
    StageNotExist(&'a str),
    StageNotExistInBusy(&'a str),
    StageNotExistInSpare(&'a str),
}

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
