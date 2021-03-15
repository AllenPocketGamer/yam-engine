use crate::{
    misc::color::{Hex, Rgba},
    nalgebra::Vector2,
};

use std::fmt;

/// Geometry representation, 32 bytes.
#[rustfmt::skip]
#[repr(C, packed(4))]
#[derive(Debug, Clone, Copy)]
pub struct Geometry {
    // type info
    pub gtype: GeometryType,    // 1 byte
    pub btype: BorderType,      // 1 byte
    pub itype: InnerType,       // 1 byte
    pub order: u8,              // 1 byte

    // decor info
    pub bcolor: Hex,            // 4 byte, border color
    pub icolor: Hex,            // 4 byte, inner color
    pub thickness: f32,         // 4 byte, border thickness

    // extra info about transformation
    pub extra: Extra,           // 16 byte
}

impl Geometry {
    pub fn new_circle(
        centra: Vector2<f32>,
        radius: f32,
        order: u8,

        btype: BorderType,
        bcolor: Hex,
        thickness: f32,
        itype: InnerType,
        icolor: Hex,
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

    pub fn new_line(
        st: Vector2<f32>,
        ed: Vector2<f32>,
        order: u8,
        btype: BorderType,
        bcolor: Hex,
        thickness: f32,
    ) -> Self {
        Self {
            gtype: GeometryType::Line,
            btype,
            itype: InnerType::None,
            order,

            bcolor,
            icolor: Rgba::BLACK.to_hex(),
            thickness,

            extra: Extra {
                point_point: (st, ed),
            },
        }
    }

    pub fn new_equilateral_triangle(
        centra: Vector2<f32>,
        radius: f32,
        angle: f32,
        order: u8,

        btype: BorderType,
        bcolor: Hex,
        thickness: f32,
        itype: InnerType,
        icolor: Hex,
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

    pub fn new_square(
        centra: Vector2<f32>,
        radius: f32,
        angle: f32,
        order: u8,

        btype: BorderType,
        bcolor: Hex,
        thickness: f32,
        itype: InnerType,
        icolor: Hex,
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
