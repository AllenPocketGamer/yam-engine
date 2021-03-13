use crate::misc::color::{Hex, Rgba};

#[repr(C, packed(4))]
#[derive(Debug, Clone, Copy)]
pub struct Geometry {
    pub icolor: Hex,
    pub bcolor: Hex,
    pub thickness: f32,

    pub gtype: GeometryType,
    pub itype: InnerType,
    pub btype: BorderType,
    _extra_info: u8,
}

impl Geometry {
    pub fn new(
        gtype: GeometryType,
        itype: InnerType,
        icolor: Hex,
        btype: BorderType,
        bcolor: Hex,
        thickness: f32,
    ) -> Self {
        Self {
            icolor,
            bcolor,
            thickness,

            gtype,
            itype,
            btype,
            _extra_info: 0,
        }
    }
}

impl Default for Geometry {
    fn default() -> Self {
        Self::new(
            GeometryType::Circle,
            InnerType::Solid,
            Rgba::CYAN.to_hex(),
            BorderType::Solid,
            Rgba::WHITE.to_hex(),
            0.1,
        )
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryType {
    Circle = 0,

    Triangle, // ⯅
    Square,   // □
    Pentagon, // ⬟
    Hexagon,  // ⎔
    FpStar,   // 🟊
    SpStar,   // 🟌
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
        assert_eq!(size_of::<Geometry>(), 16);

        assert_eq!(align_of::<GeometryType>(), 1);
        assert_eq!(size_of::<GeometryType>(), 1);

        assert_eq!(align_of::<InnerType>(), 1);
        assert_eq!(size_of::<InnerType>(), 1);

        assert_eq!(align_of::<BorderType>(), 1);
        assert_eq!(size_of::<BorderType>(), 1);
    }
}
