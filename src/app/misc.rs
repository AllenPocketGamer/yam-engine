use super::App;
use std::{
    borrow::Borrow,
    fmt::Display,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy)]
pub enum RenderFramerate {
    Low,    // 30fps
    Normal, // 60fps
    High,   // 144fps
    Custom(u32),
}

#[derive(Debug, Clone, Copy)]
pub struct Timer {
    pub target_ticks: u32,
    pub target_delta: Duration,
    pub last_tick: Instant,
    pub accumulated_delta: Duration,
    pub has_ticked: bool,
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
            target_delta: Duration::new(target_seconds, target_nanos),
            last_tick: Instant::now(),
            accumulated_delta: Duration::from_secs(0),
            has_ticked: false,
        }
    }

    pub(crate) fn update(&mut self) {
        let now = Instant::now();
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

    pub(crate) fn set_ticks_per_second(&mut self, ticks_per_second: u32) {
        let (target_seconds, target_nanos) = match ticks_per_second {
            0 => (std::u64::MAX, 0),
            1 => (1, 0),
            _ => (0, ((1.0 / ticks_per_second as f64) * 1e9) as u32),
        };
        
        self.target_ticks = ticks_per_second;
        self.target_delta = Duration::new(target_seconds, target_nanos);
    }

    pub fn next_tick_proximity(&self) -> f32 {
        let delta = self.accumulated_delta;

        self.target_ticks as f32 * (delta.as_secs() as f32 + (delta.subsec_micros() as f32 / 1_000_000.0))
    }
}

pub struct AppSettings {
    render_framerate_current: RenderFramerate,
    layer_to_frequency_current: Vec<(String, u32)>,
    commands: Vec<AppCommand>,
}

impl AppSettings {
    pub(super) fn new(app: &App) -> Self {
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

    pub(super) fn take_commands(&mut self) -> Option<Vec<AppCommand>> {
        match self.commands.len() > 0 {
            true => {
                let mut commands: Vec<AppCommand> = Vec::new();
                std::mem::swap(&mut self.commands, &mut commands);
    
                Some(commands)
            },
            false => None,
        }
    }

    pub fn get_render_framerate(&self) -> RenderFramerate {
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

        let l2f = self
            .get_l2f_mut(&layer_name)
            .expect(format!("not find update layer named: {}", layer_name).as_str());
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

pub(super) enum AppCommand {
    SetUpdateLayerFrequency(String, u32),
    SetRenderFramerate(RenderFramerate),
}
