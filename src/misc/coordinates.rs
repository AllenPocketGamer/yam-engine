use crate::nalgebra::Matrix4;

/// Record the datas about coordinate transformation.
#[derive(Debug, Clone, Copy)]
pub struct Transformation {
    pub mx_view: Matrix4<f32>,
    pub mx_proj: Matrix4<f32>,
    pub mx_viewport: Matrix4<f32>,
}

impl Default for Transformation {
    fn default() -> Self {
        Self {
            mx_view: Matrix4::identity(),
            mx_proj: Matrix4::identity(),
            mx_viewport: Matrix4::identity(),
        }
    }
}

impl Transformation {
    /// Transform points(or vectors) from `screen space` to `ndc`.
    pub fn mx_s2n(&self) -> Matrix4<f32> {
        self.mx_viewport.try_inverse().unwrap()
    }

    /// Transform points(or vectors) from `ndc` to `view space`.
    pub fn mx_n2v(&self) -> Matrix4<f32> {
        self.mx_proj.try_inverse().unwrap()
    }

    /// Transform points(or vectors) from `view space` to `world space`.
    pub fn mx_v2w(&self) -> Matrix4<f32> {
        self.mx_view.try_inverse().unwrap()
    }

    /// Transform points(or vectors) from `screen space` to `view space`.
    pub fn mx_s2v(&self) -> Matrix4<f32> {
        (self.mx_viewport * self.mx_proj).try_inverse().unwrap()
    }

    /// Transform points(or vectors) from `screen space` to `world space`.
    pub fn mx_s2w(&self) -> Matrix4<f32> {
        (self.mx_viewport * self.mx_proj * self.mx_view)
            .try_inverse()
            .unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coordinate {
    LocalSpace = 0,
    WorldSpace = 1,
    ViewSpace = 2,
    Ndc = 3,
    ScreenSpace = 4,
}
