extern crate nalgebra as na;

/// Transformation from local space to world space.
///
/// Position, rotation and scale of an entity in world space.
#[derive(Debug, Clone, Copy)]
pub struct Transform2D {
    pub position: na::Vector2<f32>,
    pub rotation: na::UnitComplex<f32>,
    pub scale: na::Vector2<f32>,
}

impl Transform2D {
    pub fn new(tx: f32, ty: f32, angle: f32, sx: f32, sy: f32) -> Self {
        Self {
            position: na::Vector2::new(tx, ty),
            rotation: na::UnitComplex::new(angle),
            scale: na::Vector2::new(sx, sy),
        }
    }

    pub fn new_with_translation(tx: f32, ty: f32) -> Self {
        Self {
            position: na::Vector2::new(tx, ty),
            rotation: na::UnitComplex::new(0.0),
            scale: na::Vector2::new(1.0, 1.0),
        }
    }

    pub fn new_with_rotation(angle: f32) -> Self {
        Self {
            position: na::Vector2::new(0.0, 0.0),
            rotation: na::UnitComplex::new(angle),
            scale: na::Vector2::new(1.0, 1.0),
        }
    }

    pub fn new_with_scale(sx: f32, sy: f32) -> Self {
        Self {
            position: na::Vector2::new(0.0, 0.0),
            rotation: na::UnitComplex::new(0.0),
            scale: na::Vector2::new(sx, sy),
        }
    }

    pub fn angle(&self) -> f32 {
        self.rotation.angle()
    }

    pub fn set_angle(&mut self, angle: f32) {
        self.rotation = na::UnitComplex::new(angle);
    }

    pub fn rotate(&mut self, delta_angle: f32) {
        self.rotation *= na::UnitComplex::new(delta_angle);
    }

    pub fn heading(&self) -> na::Vector2<f32> {
        na::Vector2::new(self.rotation.re, self.rotation.im)
    }

    pub fn set_heading(&mut self, heading: &na::Vector2<f32>) {
        let heading = heading.normalize();

        self.rotation = na::UnitComplex::from_cos_sin_unchecked(heading.x, heading.y);
    }

    pub fn to_homogeneous(&self) -> na::Matrix3<f32> {
        let scale = na::Vector2::new(
            Self::normal_or_min(self.scale.x),
            Self::normal_or_min(self.scale.y),
        );

        self.rotation
            .to_homogeneous()
            .prepend_nonuniform_scaling(&scale)
            .append_translation(&self.position)
    }

    pub fn to_homogeneous_3d(&self) -> na::Matrix4<f32> {
        let scale = na::Vector3::new(
            Self::normal_or_min(self.scale.x),
            Self::normal_or_min(self.scale.y),
            1.0,
        );

        na::UnitQuaternion::new(na::Vector3::new(0.0, 0.0, self.rotation.angle()))
            .to_homogeneous()
            .prepend_nonuniform_scaling(&scale)
            .append_translation(&na::Vector3::new(self.position.x, self.position.y, 0.0))
    }

    fn normal_or_min(num: f32) -> f32 {
        if num.is_normal() {
            num
        } else {
            f32::MIN
        }
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0, 1.0)
    }
}

unsafe impl bytemuck::Zeroable for Transform2D {}
unsafe impl bytemuck::Pod for Transform2D {}

#[cfg(test)]
mod tests {
    extern crate nalgebra as na;

    use super::Transform2D;
    use std::assert_eq;

    #[test]
    fn check_transform2d() {
        let t = Transform2D::new(1.0, 2.0, std::f32::consts::PI / 4.0, 3.0, 4.0);

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let m_t = na::Matrix3::<f32>::new(
            1.0, 0.0, 1.0,
            0.0, 1.0, 2.0,
            0.0, 0.0, 1.0,
        );

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let m_r = na::Matrix3::<f32>::new(
            0.70710677, -0.70710677, 0.0,
            0.70710677, 0.70710677, 0.0,
            0.0, 0.0, 1.0,
        );

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let m_s = na::Matrix3::<f32>::new(
            3.0, 0.0, 0.0,
            0.0, 4.0, 0.0,
            0.0, 0.0, 1.0,
        );

        assert_eq!(t.to_homogeneous(), m_t * m_r * m_s);
    }
}
