use crate::nalgebra::{Matrix3, Matrix4, UnitComplex, UnitQuaternion, Vector2, Vector3};

/// Transformation from local space to world space.
///
/// Position, rotation and scale of an entity in world space.
#[derive(Debug, Clone, Copy)]
pub struct Transform2D {
    pub position: Vector2<f32>,
    pub rotation: UnitComplex<f32>,
    pub scale: Vector2<f32>,
}

impl Transform2D {
    pub fn new(tx: f32, ty: f32, angle: f32, sx: f32, sy: f32) -> Self {
        Self {
            position: Vector2::new(tx, ty),
            rotation: UnitComplex::new(angle),
            scale: Vector2::new(sx, sy),
        }
    }

    pub fn with_position(tx: f32, ty: f32) -> Self {
        Self {
            position: Vector2::new(tx, ty),
            rotation: UnitComplex::new(0.0),
            scale: Vector2::new(1.0, 1.0),
        }
    }

    pub fn with_rotation(angle: f32) -> Self {
        Self {
            position: Vector2::new(0.0, 0.0),
            rotation: UnitComplex::new(angle),
            scale: Vector2::new(1.0, 1.0),
        }
    }

    pub fn with_scale(sx: f32, sy: f32) -> Self {
        Self {
            position: Vector2::new(0.0, 0.0),
            rotation: UnitComplex::new(0.0),
            scale: Vector2::new(sx, sy),
        }
    }

    pub fn angle(&self) -> f32 {
        self.rotation.angle()
    }

    pub fn set_angle(&mut self, angle: f32) {
        self.rotation = UnitComplex::new(angle);
    }

    pub fn rotate(&mut self, delta_angle: f32) {
        self.rotation *= UnitComplex::new(delta_angle);
    }

    pub fn heading(&self) -> Vector2<f32> {
        Vector2::new(self.rotation.re, self.rotation.im)
    }

    pub fn set_heading(&mut self, heading: &Vector2<f32>) {
        let heading = heading.normalize();

        self.rotation = UnitComplex::from_cos_sin_unchecked(heading.x, heading.y);
    }

    pub fn to_homogeneous(&self) -> Matrix3<f32> {
        let scale = Vector2::new(
            Self::normal_or_min(self.scale.x),
            Self::normal_or_min(self.scale.y),
        );

        self.rotation
            .to_homogeneous()
            .prepend_nonuniform_scaling(&scale)
            .append_translation(&self.position)
    }

    pub fn to_homogeneous_3d(&self) -> Matrix4<f32> {
        let scale = Vector3::new(
            Self::normal_or_min(self.scale.x),
            Self::normal_or_min(self.scale.y),
            1.0,
        );

        UnitQuaternion::new(Vector3::new(0.0, 0.0, self.rotation.angle()))
            .to_homogeneous()
            .prepend_nonuniform_scaling(&scale)
            .append_translation(&Vector3::new(self.position.x, self.position.y, 0.0))
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
    use crate::nalgebra::Matrix3;

    use super::Transform2D;
    use std::assert_eq;

    #[test]
    fn check_transform2d() {
        let t = Transform2D::new(1.0, 2.0, std::f32::consts::PI / 4.0, 3.0, 4.0);

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let m_t = Matrix3::<f32>::new(
            1.0, 0.0, 1.0,
            0.0, 1.0, 2.0,
            0.0, 0.0, 1.0,
        );

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let m_r = Matrix3::<f32>::new(
            0.70710677, -0.70710677, 0.0,
            0.70710677, 0.70710677, 0.0,
            0.0, 0.0, 1.0,
        );

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let m_s = Matrix3::<f32>::new(
            3.0, 0.0, 0.0,
            0.0, 4.0, 0.0,
            0.0, 0.0, 1.0,
        );

        assert_eq!(t.to_homogeneous(), m_t * m_r * m_s);
    }
}
