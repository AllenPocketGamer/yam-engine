use std::{
    collections::HashMap,
    mem::{discriminant, Discriminant},
    slice::Iter,
};

use super::app::AppStage;

pub struct WindowSettings {}

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
