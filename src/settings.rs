use std::{
    collections::HashMap,
    mem::{Discriminant, discriminant},
};

use super::app::AppStage;

pub struct Settings {
    app: AppSettings,
    window: WindowSettings,
}

impl Settings {
    pub(crate) fn new() -> Self {
        Settings {
            app: AppSettings::new(),
            window: WindowSettings::new(),
        }
    }
}

pub struct AppSettings {
    layer_to_frequency_current: Vec<(String, u32)>,
    commands: HashMap<Discriminant<AppCommand>, AppCommand>,
}

impl AppSettings {
    pub(crate) fn new() -> Self {
        todo!()
    }

    pub fn add_stage(&mut self, stage: AppStage) {
        todo!()
    }

    pub fn remove_stage(&mut self, stage: AppStage) {
        todo!()
    }

    pub fn stage(&self, stage_name: &str) -> &AppStage {
        todo!()
    }

    pub fn stage_mut(&mut self, stage_name: &str) -> &mut AppStage {
        todo!()
    }

    // pub fn stages(&self) -> impl Iterator<Item = &AppStage> {
    //     todo!()
    // }

    // pub fn stages_mut(&mut self) -> impl Iterator<Item = &AppStage> {
    //     todo!()
    // }
}

pub struct WindowSettings {

}

impl WindowSettings {
    pub(crate) fn new() -> Self {
        todo!()
    }
}

// TODO: change argument to appropriate form
// FIXME: change String to &str
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(super) enum AppCommand {
    SetUpdateLayerFrequency(String, u32),

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
