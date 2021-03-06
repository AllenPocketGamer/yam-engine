pub mod app;
pub mod components;
pub mod input;
pub mod misc;
pub mod render;
pub mod window;

// Use crate `legion` as the ecs framework of yam engine.
//
// Click [this](https://github.com/amethyst/legion) for more information.
pub extern crate legion;
// Use crate `nalgebra` as the algebra tool of yam engine.
//
// Click [this](https://github.com/dimforge/nalgebra) for more information.
pub extern crate nalgebra;

pub use app::*;
pub use components::{
    camera::Camera2D,
    geometry::{
        Assembly, BorderDecoration, BorderThickness, Geometry, Geometry1DType, Geometry2DType,
        GeometryType, InnerDecoration,
    },
    sprite::Sprite,
    time::{DiagnosticTimer, Time},
    transform::Transform2D,
    Instance,
};
pub use input::{Input, KeyCode, MouseButton};
pub use misc::color::{Hex, Rgba};
pub use window::{Fullscreen, MonitorHandle, VideoMode, Window};

const DEFAULT_WIDTH: u32 = 1280;
const DEFAULT_HEIGHT: u32 = 720;
