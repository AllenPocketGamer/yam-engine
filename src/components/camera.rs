use crate::{
    nalgebra::{Matrix4, Orthographic3},
    DEFAULT_HEIGHT, DEFAULT_WIDTH,
};

pub struct Camera2D {
    pub width: u32,
    pub height: u32,
}

impl Camera2D {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn to_orthographic(&self) -> Orthographic3<f32> {
        let half_wdith = self.width as f32 / 2.0;
        let half_height = self.height as f32 / 2.0;

        Orthographic3::new(
            -half_wdith,
            half_wdith,
            -half_height,
            half_height,
            0.0,
            256.0,
        )
    }

    pub fn to_orthographic_homogeneous(&self) -> Matrix4<f32> {
        self.to_orthographic().to_homogeneous()
    }
}

impl Default for Camera2D {
    fn default() -> Self {
        Self::new(DEFAULT_WIDTH, DEFAULT_HEIGHT)
    }
}
