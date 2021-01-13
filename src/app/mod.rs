mod misc;

pub use misc::*;

use super::window::*;

use std::{borrow::Borrow, panic};
use std::{fmt::Display};

use legion::{
    systems::{Builder, ParallelRunnable},
    Resources, Schedule, World,
};

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

    pub fn set_render_framerate(self, _framerate: RenderFramerate) -> Self {
        self
    }

    pub fn finish(mut self) -> App {
        App {
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
        self.update_layer_builders
            .iter_mut()
            .find(|builder| builder.0.borrow() == layer_name)
    }
}

pub struct App {
    startup_scheduler: Schedule,
    update_layers: Vec<UpdateLayer>,
}

impl App {
    pub fn build() -> AppBuilder {
        AppBuilder::new()
    }

    pub fn run(mut self) {
        let parse_settings = |app: &mut App, window: &mut Window, settings: &mut AppSettings| {
            if let Some(commands) = settings.take_commands() {
                for cmd in commands {
                    match cmd {
                        AppCommand::SetRenderFramerate(fr) => todo!(),
                        AppCommand::SetUpdateLayerFrequency(name, freq) => app.set_update_layer_frequency(name.as_str(), freq),
                        AppCommand::SetWindowAlwaysOnTop(always_on_top) => window.set_always_on_top(always_on_top),
                        AppCommand::SetWindowCursorGrab(grab) => window.set_cursor_grab(grab),
                        AppCommand::SetWindowFullScreen(fullscreen) => window.set_fullscreen(fullscreen),
                        AppCommand::SetWindowInnerSize(size) => window.set_inner_size(size),
                        AppCommand::SetWindowMaxInnerSize(max_size) => window.set_max_inner_sie(max_size),
                        AppCommand::SetWindowMaximized(maximized) => window.set_maximized(maximized),
                        AppCommand::SetWindowMinInnerSize(min_size) => window.set_min_inner_size(min_size),
                        AppCommand::SetWindowMinimized(minimized) => window.set_minimized(minimized),
                        AppCommand::SetWindowOuterPosition(position) => window.set_outer_position(position),
                        AppCommand::SetWindowResizable(resizable) => window.set_resizable(resizable),
                        AppCommand::SetWindowTitle(title) => window.set_title(title),
                        AppCommand::SetWindowVisible(visible) => window.set_visible(visible),
                    }
                }
            }
        };

        let mut temp_timer = Timer::new(60);

        let mut world = World::default();
        let mut resources = Resources::default();
        let mut window = Window::new().expect("unexpected error");

        let settings = AppSettings::new(&self);
        let input = Input::new();

        // insert resource to resources
        resources.insert::<AppSettings>(settings);
        resources.insert::<Input>(input);

        self.startup_scheduler.execute(&mut world, &mut resources);

        if self.update_layers.len() > 0 {
            loop {
                for update_layer in self.update_layers.iter_mut() {
                    update_layer.timer.update();

                    if update_layer.timer.tick() {
                        resources.insert::<Timer>(update_layer.timer);
                        update_layer.scheduler.execute(&mut world, &mut resources);
                    }
                }

                temp_timer.update();
                if temp_timer.tick() {
                    {
                        let settings = &mut *resources.get_mut::<AppSettings>().expect("not find AppSettings in Resources");
                        parse_settings(&mut self, &mut window, settings);
                    }

                    {
                        let input = &mut *resources.get_mut::<Input>().expect("not find Input in Resources");
                        window.run_return(input);
                    }
                }
            }
        }
    }

    fn set_update_layer_frequency<T>(&mut self, layer_name: &T, target_frequency: u32)
    where
        T: Eq + ?Sized,
        String: Borrow<T>,
    {
        let update_layer = self
            .update_layers
            .iter_mut()
            .find(|ul| ul.name.borrow() == layer_name)
            .expect("inner error");
        update_layer.timer.set_ticks_per_second(target_frequency);
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
