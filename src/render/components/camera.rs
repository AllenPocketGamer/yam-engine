extern crate nalgebra as na;

pub struct Camera2D {
    orth: na::Orthographic3<f32>,
}

impl Camera2D {
    fn new() -> Self {
        todo!()
    }
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            orth: na::Orthographic3::new(-10.0, 10.0, -10.0, 10.0, 0.0, -10.0),
        }
    }
}
