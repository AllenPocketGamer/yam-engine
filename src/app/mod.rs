use std::{borrow::Borrow, panic};
use std::{fmt::Display, time};

use legion::{
    systems::{Builder, ParallelRunnable},
    Resources, Schedule, World,
};

#[derive(Debug, Clone, Copy)]
pub enum RenderFramerate {
    Low,    // 30fps
    Normal, // 60fps
    High,   // 144fps
    Custom(u32),
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

        self.target_ticks as f32 * (delta.as_secs() as f32 + (delta.subsec_micros() as f32 / 1_000_000.0))
    }
}

pub struct AppBuilder {
    startup_builder: Builder,
    update_layer_builders: Vec<(String, u32, Builder, u32)>,
}

impl AppBuilder {
    fn new() -> Self {
        AppBuilder {
            startup_builder: Builder::default(),
            update_layer_builders: Vec::new(),
        }
    }

    pub fn add_startup_system<T: ParallelRunnable + 'static>(mut self, system: T) -> Self {
        self.startup_builder.add_system(system);

        self
    }

    pub fn add_update_system<T, U>(mut self, system: T, layer_name: &U) -> Self
    where
        T: ParallelRunnable + 'static,
        String: Borrow<U>,
        U: Eq + ?Sized + Display,
    {
        let update_builder = self.get_update_builder(layer_name);

        if let Some(value) = update_builder {
            value.2.add_system(system);
            value.3 += 1;
        } else {
            panic!(format!("not find update layer named: {}", &layer_name));
        }

        self
    }

    pub fn add_layer_to_update(mut self, layer_name: impl Into<String>, frequency: u32) -> Self {
        let layer_name: String = layer_name.into();

        if self.update_layer_builders.len() >= 8 {
            panic!("no more space to place update layer!");
        }

        if self.get_update_builder(&layer_name).is_some() {
            panic!(format!("already had layer named: {}!", layer_name));
        }

        self.update_layer_builders.push((layer_name, frequency, Builder::default(), 0));

        self
    }

    pub fn set_render_framerate(mut self, framerate: RenderFramerate) -> Self {
        self
    }

    pub fn finish(mut self) -> App {
        App {
            world: World::default(),
            resources: Resources::default(),
            startup_scheduler: self.startup_builder.build(),
            update_layers: self
                .update_layer_builders
                .into_iter()
                .filter(|builder| builder.3 != 0)
                .map(|builder| UpdateLayer::from((builder.0, builder.1, builder.2)))
                .collect(),
        }
    }

    fn get_update_builder<T>(&mut self, layer_name: &T) -> Option<&mut (String, u32, Builder, u32)>
    where
        T: Eq + ?Sized,
        String: Borrow<T>,
    {
        self.update_layer_builders.iter_mut().find(|builder| builder.0.borrow() == layer_name)
    }
}

pub struct App {
    world: World,
    resources: Resources,
    startup_scheduler: Schedule,
    update_layers: Vec<UpdateLayer>,
}

impl App {
    pub fn build() -> AppBuilder {
        AppBuilder::new()
    }

    pub fn run(mut self) {
        let world = &mut self.world;
        let resources = &mut self.resources;

        // insert resource to resources
        let kkk = String::from("Hello World");
        let kkk = kkk.as_str();
        let foo = Foo { bar: kkk };
        resources.insert::<&Foo>(&Foo { bar: "Hell" });

        self.startup_scheduler.execute(world, resources);

        if self.update_layers.len() > 0 {
            loop {
                for update_layer in self.update_layers.iter_mut() {
                    update_layer.timer.update();

                    if update_layer.timer.tick() {
                        update_layer.scheduler.execute(world, resources);
                    }
                }
            }
        }
    }
}

pub struct AppSettings {
    render_framerate_current: RenderFramerate,
    layer_to_frequency_current: Vec<(String, u32)>,
    commands: Vec<AppCommand>,
}

impl AppSettings {
    fn new(app: &App) -> Self {
        AppSettings {
            render_framerate_current: RenderFramerate::Normal, // TODO fix it after implement render
            layer_to_frequency_current: app
                .update_layers
                .iter()
                .map(|update_layer| (update_layer.name.clone(), update_layer.timer.target_ticks))
                .collect(),
            commands: Vec::new(),
        }
    }

    fn update(&mut self, app: &mut App) {

    }

    pub fn get_render_framerate(&self) -> u32 {
        self.render_framerate_current
    }

    pub fn get_update_layer_frequency<T>(&self, layer_name: &T) -> u32
    where
        T: Eq + Display + ?Sized,
        String: Borrow<T>,
    {
        self.get_l2f(layer_name)
            .expect(format!("not find update layer named: {}", layer_name).as_str())
            .1
    }

    pub fn set_render_framerate(&mut self, target_framerate: RenderFramerate) {
        self.render_framerate_current = target_framerate;
        self.commands.push(AppCommand::SetRenderFramerate(target_framerate));
    }

    pub fn set_update_layer_frequency(&mut self, layer_name: impl Into<String>, target_frequency: u32) {
        let layer_name = layer_name.into();

        let l2f = self.get_l2f_mut(&layer_name).expect(format!("not find update layer named: {}", layer_name).as_str());
        l2f.1 = target_frequency;
        
        self.commands.push(AppCommand::SetUpdateLayerFrequency(layer_name, target_frequency))
    }

    fn get_l2f<T>(&self, layer_name: &T) -> Option<&(String, u32)>
    where
        T: Eq + ?Sized,
        String: Borrow<T>,
    {
        self.layer_to_frequency_current.iter().find(|l2f| l2f.0.borrow() == layer_name)
    }

    fn get_l2f_mut<T>(&mut self, layer_name: &T) -> Option<&mut (String, u32)>
    where
        T: Eq + ?Sized,
        String: Borrow<T>,
    {
        self.layer_to_frequency_current.iter_mut().find(|l2f| l2f.0.borrow() == layer_name)
    }
}

struct UpdateLayer {
    name: String,
    timer: Timer,
    scheduler: Schedule,
}

impl UpdateLayer {
    fn from(mut update_layer_builder: (String, u32, Builder)) -> Self {
        UpdateLayer {
            name: update_layer_builder.0,
            timer: Timer::new(update_layer_builder.1),
            scheduler: update_layer_builder.2.build(),
        }
    }
}

enum AppCommand {
    SetUpdateLayerFrequency(String, u32),
    SetRenderFramerate(RenderFramerate),
}

struct Foo<'a> {
    bar: &'a str,
}
