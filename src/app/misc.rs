use super::App;
use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::{Display, Formatter, *},
    mem::{discriminant, Discriminant},
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderFramerate {
    Low,    // 30fps
    Normal, // 60fps
    High,   // 144fps
    Custom(u32),
}

impl Default for RenderFramerate {
    fn default() -> Self {
        RenderFramerate::Normal
    }
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

impl Display for Timer {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "[interval]: {}ms, [delta]: {}ms [ticked?]: {}",
            self.target_delta.as_millis(),
            self.accumulated_delta.as_millis(),
            self.has_ticked
        )
    }
}

pub struct AppSettings {
    render_framerate_current: RenderFramerate,
    layer_to_frequency_current: Vec<(String, u32)>,
    
    is_fullscreen: bool,
    inner_position: (i32, i32),
    inner_size: (u32, u32),
    outer_position: (i32, i32),
    outer_size: (u32, u32),
    scale_factor: f64,
    
    commands: HashMap<Discriminant<AppCommand>, AppCommand>,
}

impl AppSettings {
    pub(super) fn new(app: &App) -> Self {
        AppSettings {
            render_framerate_current: Default::default(),
            layer_to_frequency_current: app
                .update_layers
                .iter()
                .map(|update_layer| (update_layer.name.clone(), update_layer.timer.target_ticks))
                .collect(),

            is_fullscreen: Default::default(),
            inner_position: Default::default(),
            inner_size: Default::default(),
            outer_position: Default::default(),
            outer_size: Default::default(),
            scale_factor: Default::default(),

            commands: HashMap::new(),
        }
    }

    pub(super) fn take_commands(&mut self) -> Option<Vec<AppCommand>> {
        if self.commands.len() > 0 {
            let result: Option<Vec<AppCommand>> = Some(self.commands.iter().map(|(_, v)| (*v).clone()).collect());
            self.commands.clear();

            result
        } else {
            None
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
    
    pub fn is_fullscreen(&self) -> bool {
        self.is_fullscreen
    }

    pub fn get_inner_position(&self) -> (i32, i32) {
        self.inner_position
    }

    pub fn get_inner_size(&self) -> (u32, u32) {
        self.inner_size
    }

    pub fn get_outer_position(&self) -> (i32, i32) {
        self.outer_position
    }

    pub fn get_outer_size(&self) -> (u32, u32) {
        self.outer_size
    }

    pub fn get_scale_factor(&self) -> f64 {
        self.scale_factor
    }
    
    pub fn iter_update_layer_to_frequency(&self) -> impl Iterator<Item = (&str, u32)> {
        self.layer_to_frequency_current
            .iter()
            .map(|(name, frequency)| (name.as_ref(), *frequency))
    }

    pub fn set_render_framerate(&mut self, target_framerate: RenderFramerate) {
        self.insert_cmd(AppCommand::SetRenderFramerate(target_framerate));
    }

    // FIXME: change layer_name type to &str
    pub fn set_update_layer_frequency(&mut self, layer_name: impl Into<String>, target_frequency: u32) {
        let layer_name = layer_name.into();
        self.insert_cmd(AppCommand::SetUpdateLayerFrequency(layer_name, target_frequency));
    }


    pub fn set_always_on_top(&mut self, always_on_top: bool) {
        self.insert_cmd(AppCommand::SetWindowAlwaysOnTop(always_on_top));
    }

    pub fn set_cursor_grab(&mut self, grab: bool) {
        self.insert_cmd(AppCommand::SetWindowCursorGrab(grab));
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        self.insert_cmd(AppCommand::SetWindowFullScreen(fullscreen));
    }

    pub fn set_inner_size(&mut self, size: (u32, u32)) {
        self.insert_cmd(AppCommand::SetWindowInnerSize(size));
    }

    pub fn set_max_inner_sie(&mut self, max_size: (u32, u32)) {
        self.insert_cmd(AppCommand::SetWindowMaxInnerSize(max_size));
    }

    pub fn set_maximized(&mut self, maximized: bool) {
        self.insert_cmd(AppCommand::SetWindowMaximized(maximized));
    }

    pub fn set_min_inner_size(&mut self, min_size: (u32, u32)) {
        self.insert_cmd(AppCommand::SetWindowMinInnerSize(min_size));
    }

    pub fn set_minimized(&mut self, minimized: bool) {
        self.insert_cmd(AppCommand::SetWindowMinimized(minimized));
    }

    pub fn set_outer_position(&mut self, position: (i32, i32)) {
        self.insert_cmd(AppCommand::SetWindowOuterPosition(position));
    }

    pub fn set_resizable(&mut self, resizable: bool) {
        self.insert_cmd(AppCommand::SetWindowResizable(resizable));
    }

    pub fn set_title(&mut self, title: String) {
        self.insert_cmd(AppCommand::SetWindowTitle(title));
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.insert_cmd(AppCommand::SetWindowVisible(visible));
    }

    // TODO: to understand #[inline] macro
    fn insert_cmd(&mut self, cmd: AppCommand) {
        self.commands.insert(discriminant(&cmd), cmd);
    }
    
    fn get_l2f<T>(&self, layer_name: &T) -> Option<&(String, u32)>
    where
        T: Eq + ?Sized,
        String: Borrow<T>,
    {
        self.layer_to_frequency_current.iter().find(|l2f| l2f.0.borrow() == layer_name)
    }
}

// TODO: change argument to appropriate form
// FIXME: change String to &str
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(super) enum AppCommand {
    SetUpdateLayerFrequency(String, u32),
    SetRenderFramerate(RenderFramerate),

    SetWindowAlwaysOnTop(bool),
    SetWindowCursorGrab(bool),
    SetWindowFullScreen(bool),
    SetWindowInnerSize((u32, u32)),
    SetWindowMaxInnerSize((u32, u32)),
    SetWindowMaximized(bool),
    SetWindowMinInnerSize((u32, u32)),
    SetWindowMinimized(bool),
    SetWindowOuterPosition((i32, i32)),
    SetWindowResizable(bool),
    SetWindowTitle(String),
    SetWindowVisible(bool),
}
