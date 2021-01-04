pub use legion::*;
use storage::IntoComponentSource;
use systems::{Builder, ParallelRunnable, Resource};

pub enum Stage {
    START,
    UPDATE,
}
#[derive(Default)]
pub struct App {
    world: World,
    resources: Resources,
    system_builder_in_start_stage: Builder,
    system_builder_in_update_stage: Builder,
}

impl App {
    pub fn add_entity<T>(&mut self, components: T) -> Entity
    where
        Option<T>: IntoComponentSource,
    {
        self.world.push(components)
    }

    pub fn add_entities(&mut self, components: impl IntoComponentSource) -> &[Entity] {
        self.world.extend(components)
    }

    pub fn add_resource(&mut self, resource: impl Resource) {
        self.resources.insert(resource);
    }

    pub fn add_system<T: ParallelRunnable + 'static>(&mut self, stage: Stage, system: T) {
        match stage {
            Stage::START => self.system_builder_in_start_stage.add_system(system),
            Stage::UPDATE => self.system_builder_in_update_stage.add_system(system),
        };
    }

    pub fn run(mut self) {
        let mut schedule_in_start = self.system_builder_in_start_stage.build();
        let mut schedule_in_update = self.system_builder_in_update_stage.build();

        schedule_in_start.execute(&mut self.world, &mut self.resources);

        loop {
            schedule_in_update.execute(&mut self.world, &mut self.resources);
        }
    }
}