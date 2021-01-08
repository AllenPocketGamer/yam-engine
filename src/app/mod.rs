use std::panic;
use std::time;

use legion::{
    systems::{Builder, ParallelRunnable},
    Resources, Schedule, World,
};

#[derive(Debug, Clone, Copy)]
pub enum RenderFramerate {
    LOW,    // 30fps
    NORMAL, // 60fps
    HIGH,   // 144fps
    CUSTOM(u32),
}

#[derive(Debug)]
pub struct Timer {
    pub target_ticks: u32,
    pub(crate) target_delta: time::Duration,
    pub last_tick: time::Instant,
    pub accumulated_delta: time::Duration,
    pub(crate) has_ticked: bool,
}

impl Timer {
    pub(crate) fn new(ticks_per_second: u32) -> Timer {
        let (target_seconds, target_nanos) = match ticks_per_second {
            0 => (std::u64::MAX, 0),
            1 => (1, 0),
            _ => (0, ((1.0 / ticks_per_second as f64) * 1e9) as u32),
        };

        Timer {
            target_ticks: ticks_per_second,
            target_delta: time::Duration::new(target_seconds, target_nanos),
            last_tick: time::Instant::now(),
            accumulated_delta: time::Duration::from_secs(0),
            has_ticked: false,
        }
    }

    pub(crate) fn update(&mut self) {
        let now = time::Instant::now();
        let diff = now - self.last_tick;

        self.last_tick = now;
        self.accumulated_delta += diff;
        self.has_ticked = false;
    }

    pub(crate) fn tick(&mut self) -> bool {
        if self.accumulated_delta >= self.target_delta {
            self.accumulated_delta -= self.target_delta;
            self.has_ticked = true;

            true
        } else {
            false
        }
    }

    pub fn next_tick_proximity(&self) -> f32 {
        let delta = self.accumulated_delta;

        self.target_ticks as f32
            * (delta.as_secs() as f32 + (delta.subsec_micros() as f32 / 1_000_000.0))
    }
}

pub struct AppBuilder<'a> {
    startup_builder: Builder,
    update_layer_builders: Vec<(&'a str, u32, Builder, u32)>,
}

impl<'a> AppBuilder<'a> {
    pub fn build() -> Self {
        AppBuilder {
            startup_builder: Builder::default(),
            update_layer_builders: Vec::new(),
        }
    }

    pub fn add_startup_system<T: ParallelRunnable + 'static>(mut self, system: T) -> Self {
        self.startup_builder.add_system(system);

        self
    }

    pub fn add_update_system<T: ParallelRunnable + 'static>(
        mut self,
        system: T,
        layer_name: &'a str,
    ) -> Self {
        let update_builder = self.get_update_builder(layer_name);

        if let Some(value) = update_builder {
            value.2.add_system(system);
            value.3 += 1;
        } else {
            panic!(format!("not find update layer named: {}", layer_name));
        }

        self
    }

    pub fn add_layer_to_update(mut self, layer_name: &'a str, frequency: u32) -> Self {
        if self.update_layer_builders.len() >= 8 {
            panic!("no more space to place update layer!");
        }

        if self.get_update_builder(layer_name).is_some() {
            panic!(format!("already had layer named: {}!", layer_name));
        }

        self.update_layer_builders
            .push((layer_name, frequency, Builder::default(), 0));

        self
    }

    pub fn set_render_framerate(mut self, framerate: RenderFramerate) -> Self {
        self
    }

    pub fn run(mut self) {
        let mut world = World::default();
        let mut resources = Resources::default();
        
        let mut startup_scheduler = self.startup_builder.build();

        let mut update_layers: Vec<UpdateLayer> = self
            .update_layer_builders
            .into_iter()
            .filter(|builder| builder.3 != 0)
            .map(|builder| UpdateLayer::from((builder.0, builder.1, builder.2)))
            .collect();

        // run systems
        startup_scheduler.execute(&mut world, &mut resources);
        if update_layers.len() > 0 {
            loop {
                for update_layer in update_layers.iter_mut() {
                    update_layer.timer.update();

                    if update_layer.timer.tick() {
                        update_layer.scheduler.execute(&mut world, &mut resources);
                    }
                }
            }
        }
    }

    fn get_update_builder(
        &mut self,
        layer_name: &'a str,
    ) -> Option<&mut (&'a str, u32, Builder, u32)> {
        self.update_layer_builders
            .iter_mut()
            .find(|builder| builder.0 == layer_name)
    }
}

struct UpdateLayer<'a> {
    name: &'a str,
    timer: Timer,
    scheduler: Schedule,
}

impl<'a> UpdateLayer<'a> {
    fn from(mut update_layer_builder: (&'a str, u32, Builder)) -> Self {
        UpdateLayer {
            name: update_layer_builder.0,
            timer: Timer::new(update_layer_builder.1),
            scheduler: update_layer_builder.2.build(),
        }
    }
}
