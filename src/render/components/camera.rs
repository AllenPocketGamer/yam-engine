extern crate nalgebra as na;

pub struct Camera2D {
    orth: na::Orthographic3<f32>,
}

impl Camera2D {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            orth: na::Orthographic3::<f32>::new(
                -width / 2.0,
                width / 2.0,
                -height / 2.0,
                height / 2.0,
                0.0,
                -10.0,
            ),
        }
    }

    pub fn size(&self) -> (f32, f32) {
        (self.orth.left().abs() * 2.0, self.orth.top().abs() * 2.0)
    }

    pub fn aspect_ratio(&self) -> f32 {
        let (width, height) = self.size();
        width / height
    }

    pub fn to_homogeneous(&self) -> na::Matrix4<f32> {
        self.orth.to_homogeneous()
    }
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            orth: na::Orthographic3::new(-512.0, 512.0, -384.0, 384.0, 0.0, -10.0),
        }
    }
}
