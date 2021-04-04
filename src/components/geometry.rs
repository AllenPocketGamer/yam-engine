use crate::{
    misc::color::{Hex, Rgba},
    nalgebra::Vector2,
};

use std::fmt;

pub type Assembly = Vec<Geometry>;

/// Geometry representation, 32 bytes.
#[rustfmt::skip]
#[repr(C, packed(4))]
#[derive(Debug, Clone, Copy)]
pub struct Geometry {
    // 0-7bit   : geometry type
    // 8-15bit  : border type
    // 16-23bit : inner type
    // 24-31bit : order
    pub types: u32,             // 4bytes

    // decor info
    pub bcolor: Hex,            // 4 bytes, border color
    pub icolor: Hex,            // 4 bytes, inner color
    pub thickness: f32,         // 4 bytes, border thickness

    // extra info about transformation
    pub extras: Extra,          // 16 bytes
}

impl Geometry {
    const fn zip(gtype: GeometryType, btype: BorderType, itype: InnerType, order: u8) -> u32 {
        let gtype = gtype as u32;
        let btype = btype as u32;
        let itype = itype as u32;
        let order = order as u32;

        (gtype << 24) + (btype << 16) + (itype << 8) + order
    }

    pub fn new(
        gtype: GeometryType,
        extras: Extra,

        btype: BorderType,
        bcolor: Rgba,
        thickness: f32,

        itype: InnerType,
        icolor: Rgba,
        order: u8,
    ) -> Self {
        Self {
            types: Self::zip(gtype, btype, itype, order),

            thickness,
            bcolor: bcolor.to_hex(),
            icolor: icolor.to_hex(),

            extras,
        }
    }

    pub fn circle_with_style(
        centra: Vector2<f32>,
        length: f32,
        angle: f32,
        order: u8,

        btype: BorderType,
        bcolor: Rgba,
        thickness: f32,
        itype: InnerType,
        icolor: Rgba,
    ) -> Self {
        Self {
            types: Self::zip(GeometryType::Circle, btype, itype, order),

            bcolor: bcolor.to_hex(),
            icolor: icolor.to_hex(),
            thickness,

            extras: Extra {
                cla: (centra, length, angle),
            },
        }
    }

    pub fn new_circle(centra: Vector2<f32>, length: f32, angle: f32, order: u8) -> Self {
        Self::circle_with_style(
            centra,
            length,
            angle,
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
            types: Self::zip(GeometryType::Line, btype, InnerType::None, order),

            bcolor: bcolor.to_hex(),
            icolor: Rgba::BLACK.to_hex(),
            thickness,

            extras: Extra {
                p2p: (st, ed),
            },
        }
    }

    pub fn new_line(st: Vector2<f32>, ed: Vector2<f32>, order: u8) -> Self {
        Self::line_with_style(st, ed, order, BorderType::Solid, Rgba::WHITE, 0.1)
    }

    pub fn triangle_with_style(
        centra: Vector2<f32>,
        length: f32,
        angle: f32,
        order: u8,

        btype: BorderType,
        bcolor: Rgba,
        thickness: f32,
        itype: InnerType,
        icolor: Rgba,
    ) -> Self {
        Self {
            types: Self::zip(GeometryType::ETriangle, btype, itype, order),

            bcolor: bcolor.to_hex(),
            icolor: icolor.to_hex(),
            thickness,

            extras: Extra {
                cla: (centra, length, angle),
            },
        }
    }

    pub fn new_triangle(centra: Vector2<f32>, length: f32, angle: f32, order: u8) -> Self {
        Self::triangle_with_style(
            centra,
            length,
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
        length: f32,
        angle: f32,
        order: u8,

        btype: BorderType,
        bcolor: Rgba,
        thickness: f32,
        itype: InnerType,
        icolor: Rgba,
    ) -> Self {
        Self {
            types: Self::zip(GeometryType::Square, btype, itype, order),

            bcolor: bcolor.to_hex(),
            icolor: icolor.to_hex(),
            thickness,

            extras: Extra {
                cla: (centra, length, angle),
            },
        }
    }

    pub fn new_square(centra: Vector2<f32>, length: f32, angle: f32, order: u8) -> Self {
        Self::square_with_style(
            centra,
            length,
            angle,
            order,
            BorderType::None,
            Rgba::WHITE,
            0.1,
            InnerType::Solid,
            Rgba::WHITE,
        )
    }

    pub fn pentagon_with_style(
        centra: Vector2<f32>,
        length: f32,
        angle: f32,
        order: u8,

        btype: BorderType,
        bcolor: Rgba,
        thickness: f32,
        itype: InnerType,
        icolor: Rgba,
    ) -> Self {
        Self {
            types: Self::zip(GeometryType::Pentagon, btype, itype, order),

            bcolor: bcolor.to_hex(),
            icolor: icolor.to_hex(),
            thickness,

            extras: Extra {
                cla: (centra, length, angle),
            },
        }
    }

    pub fn new_pentagon(centra: Vector2<f32>, length: f32, angle: f32, order: u8) -> Self {
        Self::pentagon_with_style(
            centra,
            length,
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

// TODO: 太丑了, 尝试改改吧!
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub union Extra {
    // centra(Vector2<f32>) + length(f32) + angle(around centra)(f32),
    //
    // length represent the side length of quad.
    pub cla: (Vector2<f32>, f32, f32),
    // point_a(Vector2<f32>) + point_b(Vector2<f32>)
    pub p2p: (Vector2<f32>, Vector2<f32>),
}

impl Extra {
    pub fn new_cla(centra: Vector2<f32>, angle: f32, side_length: f32) -> Self {
        Self {
            cla: (centra, angle, side_length)
        }
    }

    pub fn new_pp(pa: Vector2<f32>, pb: Vector2<f32>) -> Self {
        Self {
            p2p: (pa, pb)
        }
    }
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
    Pentagon,  // ⬟
    Hexagon,   // ⎔
    Octogon,
    Hexagram,
    StarFive,
    Heart,
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
