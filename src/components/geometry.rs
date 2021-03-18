use crate::{misc::color::Rgba, nalgebra::Vector2};

use std::fmt;

pub type Assembly = Vec<Geometry>;

/// Geometry representation, 32 bytes.
#[rustfmt::skip]
#[repr(C, packed(4))]
#[derive(Debug, Clone, Copy)]
pub struct Geometry {
    // type info
    pub gtype: GeometryType,    // 1 bytes
    pub btype: BorderType,      // 1 bytes
    pub itype: InnerType,       // 1 bytes
    pub order: u8,              // 1 bytes

    // decor info
    pub bcolor: Rgba,           // 4 bytes, border color
    pub icolor: Rgba,           // 4 bytes, inner color
    pub thickness: f32,         // 4 bytes, border thickness

    // extra info about transformation
    pub extra: Extra,           // 16 bytes
}

impl Geometry {
    pub fn circle_with_style(
        centra: Vector2<f32>,
        radius: f32,
        order: u8,

        btype: BorderType,
        bcolor: Rgba,
        thickness: f32,
        itype: InnerType,
        icolor: Rgba,
    ) -> Self {
        Self {
            gtype: GeometryType::Circle,
            btype,
            itype,
            order,

            bcolor,
            icolor,
            thickness,

            extra: Extra {
                centra_radius_angle: (centra, radius, 0.0),
            },
        }
    }

    pub fn new_circle(centra: Vector2<f32>, radius: f32, order: u8) -> Self {
        Self::circle_with_style(
            centra,
            radius,
            order,
            BorderType::None,
            Rgba::WHITE,
            0.1,
            InnerType::Solid,
            Rgba::WHITE,
        )
    }

    pub fn line_with_style(
        st: Vector2<f32>,
        ed: Vector2<f32>,
        order: u8,
        btype: BorderType,
        bcolor: Rgba,
        thickness: f32,
    ) -> Self {
        Self {
            gtype: GeometryType::Line,
            btype,
            itype: InnerType::None,
            order,

            bcolor,
            icolor: Rgba::BLACK,
            thickness,

            extra: Extra {
                point_point: (st, ed),
            },
        }
    }

    pub fn new_line(st: Vector2<f32>, ed: Vector2<f32>, order: u8) -> Self {
        Self::line_with_style(st, ed, order, BorderType::Solid, Rgba::WHITE, 0.1)
    }

    pub fn triangle_with_style(
        centra: Vector2<f32>,
        radius: f32,
        angle: f32,
        order: u8,

        btype: BorderType,
        bcolor: Rgba,
        thickness: f32,
        itype: InnerType,
        icolor: Rgba,
    ) -> Self {
        Self {
            gtype: GeometryType::ETriangle,
            btype,
            itype,
            order,

            bcolor,
            icolor,
            thickness,

            extra: Extra {
                centra_radius_angle: (centra, radius, angle),
            },
        }
    }

    pub fn new_triangle(centra: Vector2<f32>, radius: f32, angle: f32, order: u8) -> Self {
        Self::triangle_with_style(
            centra,
            radius,
            angle,
            order,
            BorderType::None,
            Rgba::WHITE,
            0.1,
            InnerType::Solid,
            Rgba::WHITE,
        )
    }

    pub fn square_with_style(
        centra: Vector2<f32>,
        radius: f32,
        angle: f32,
        order: u8,

        btype: BorderType,
        bcolor: Rgba,
        thickness: f32,
        itype: InnerType,
        icolor: Rgba,
    ) -> Self {
        Self {
            gtype: GeometryType::Square,
            btype,
            itype,
            order,

            bcolor,
            icolor,
            thickness,

            extra: Extra {
                centra_radius_angle: (centra, radius, angle),
            },
        }
    }

    pub fn new_square(centra: Vector2<f32>, radius: f32, angle: f32, order: u8) -> Self {
        Self::square_with_style(
            centra,
            radius,
            angle,
            order,
            BorderType::None,
            Rgba::WHITE,
            0.1,
            InnerType::Solid,
            Rgba::WHITE,
        )
    }
}

#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub union Extra {
    // centra(Vector2<f32>) + radius(f32) + angle(around centra)(f32),
    centra_radius_angle: (Vector2<f32>, f32, f32),
    // point_a(Vector2<f32>) + point_b(Vector2<f32>)
    point_point: (Vector2<f32>, Vector2<f32>),
}

impl fmt::Debug for Extra {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "no impl now!")
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryType {
    Circle = 0,

    Line,
    ETriangle, // ⯅
    Square,    // □
               // Pentagon, // ⬟
               // Hexagon,  // ⎔
               // FpStar,   // 🟊
               // SpStar,   // 🟌
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderType {
    None = 0,
    Solid,   // ────
    Dash,    // ----
    DynDash, // ----     (will move)
    Navi,    // ▸▸▸▸
    DynNavi, // ▸▸▸▸    (will move)
    Warn,    // ////
    DynWarn, // ////     (will move)
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InnerType {
    None = 0,
    Solid,     // ██
    Dither,    // ▒▒
    DynDither, // ▒▒   (will move)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::mem::{align_of, size_of};

    #[test]
    fn test_type_layout() {
        assert_eq!(align_of::<Geometry>(), 4);
        assert_eq!(size_of::<Geometry>(), 32);

        assert_eq!(align_of::<Extra>(), 4);
        assert_eq!(size_of::<Extra>(), 16);

        assert_eq!(align_of::<GeometryType>(), 1);
        assert_eq!(size_of::<GeometryType>(), 1);

        assert_eq!(align_of::<InnerType>(), 1);
        assert_eq!(size_of::<InnerType>(), 1);

        assert_eq!(align_of::<BorderType>(), 1);
        assert_eq!(size_of::<BorderType>(), 1);
    }
}
