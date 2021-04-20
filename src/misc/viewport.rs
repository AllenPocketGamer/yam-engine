use crate::nalgebra::Matrix4;

#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,

    pub w: f32,
    pub h: f32,

    pub min_depth: f32,
    pub max_depth: f32,
}

impl Viewport {
    pub fn new_in_screen(width: f32, height: f32, aspect_ratio: f32) -> Self {
        let screen_ratio = width / height;

        let (x, y, width, height) = if aspect_ratio <= screen_ratio {
            (
                (width - aspect_ratio * height) / 2.0,
                0f32,
                aspect_ratio * height,
                height,
            )
        } else {
            (
                0f32,
                (height - width / aspect_ratio) / 2.0,
                width,
                width / aspect_ratio,
            )
        };

        Self {
            x,
            y,
            w: width,
            h: height,
            min_depth: 0.0,
            max_depth: 1.0,
        }
    }

    /// Transform point from NDC to screen space
    ///
    /// x_ss = (x_ndc + 1) / 2 * width + vp.x        , x_ndc ∈ [-1, 1]
    /// y_ss = (1 - y_ndc) / 2 * height + vp.z       , y_ndc ∈ [-1, 1]
    /// z_ss = (far - near) * z_ndc + near           , z_ndc ∈ [+0, 1]
    pub fn to_homogeneous_3d(&self) -> Matrix4<f32> {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        Matrix4::new(
            0.5 * self.w,   0.0,                0.0,                                0.5 * self.w + self.x,
            0.0,            -0.5 * self.h,      0.0,                                0.5 * self.h + self.y,
            0.0,            0.0,                self.max_depth - self.min_depth,    self.min_depth,
            0.0,            0.0,                0.0,                                1.0,
        )
    }
}
