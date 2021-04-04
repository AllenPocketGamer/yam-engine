use crate::{
    misc::color::{Hex, Rgba},
    nalgebra::Vector2,
};

pub type Assembly = Vec<Geometry2D>;

/// (32bytes)Geometry2D representation.
#[rustfmt::skip]
#[repr(C, packed(4))]
#[derive(Debug, Clone, Copy)]
pub struct Geometry2D {
    /// 0-7bit   : geometry type
    ///
    /// 8-15bit  : border type
    ///
    /// 16-23bit : inner type
    ///
    /// 24-31bit : order
    datas: u32,

    /// (4bytes)The border color.
    pub bcolor: Hex,
    /// (4bytes)The inner color.
    pub icolor: Hex,
    /// (4bytes)Positive represents thickness in `screen space`,
    /// Negative represents thickness in `local space`.
    pub thickness: f32,

    /// (8bytes)The quad centra in `local space`.
    pub position: Vector2<f32>,
    /// (4bytes)The quad angle in `local space`.
    pub angle: f32,
    /// (4bytes)The quad side length in `local space`.
    pub size: f32,
}

impl Geometry2D {
    const fn zip(
        gtype: Geometry2DType,
        bdeco: BorderDecoration,
        ideco: InnerDecoration,
        order: u8,
    ) -> u32 {
        let gtype = gtype as u32;
        let bdeco = bdeco as u32;
        let ideco = ideco as u32;
        let order = order as u32;

        (gtype << 24) + (bdeco << 16) + (ideco << 8) + order
    }

    const fn zip_u8(gtype: u8, bdeco: u8, ideco: u8, order: u8) -> u32 {
        ((gtype as u32) << 24) + ((bdeco as u32) << 16) + ((ideco as u32) << 8) + order as u32
    }

    const fn unzip(datas: u32) -> [u8; 4] {
        let gtype = (datas >> 24) as u8;
        let bdeco = (datas >> 16 & 0xFF) as u8;
        let ideco = (datas >> 8 & 0xFF) as u8;
        let order = (datas & 0xFF) as u8;

        [gtype, bdeco, ideco, order]
    }

    pub const fn new(
        gtype: Geometry2DType,
        bdeco: BorderDecoration,
        bcolor: Rgba,
        thickness: f32,
        ideco: InnerDecoration,
        icolor: Rgba,
        order: u8,
        position: Vector2<f32>,
        angle: f32,
        size: f32,
    ) -> Self {
        Self {
            datas: Self::zip(gtype, bdeco, ideco, order),
            thickness,
            bcolor: bcolor.to_hex(),
            icolor: icolor.to_hex(),

            position,
            angle,
            size,
        }
    }

    pub fn geometry_type(&self) -> Geometry2DType {
        unsafe { std::mem::transmute(Self::unzip(self.datas)[0]) }
    }

    pub fn border_decoration(&self) -> BorderDecoration {
        unsafe { std::mem::transmute(Self::unzip(self.datas)[1]) }
    }

    pub fn inner_decoration(&self) -> InnerDecoration {
        unsafe { std::mem::transmute(Self::unzip(self.datas)[2]) }
    }

    pub fn order(&self) -> u8 {
        Self::unzip(self.datas)[3]
    }

    pub fn set_geometry_type(&mut self, gtype: Geometry2DType) {
        let datas = Self::unzip(self.datas);
        self.datas = Self::zip_u8(gtype as u8, datas[1], datas[2], datas[3]);
    }

    pub fn set_border_decoration(&mut self, bdeco: BorderDecoration) {
        let datas = Self::unzip(self.datas);
        self.datas = Self::zip_u8(datas[0], bdeco as u8, datas[2], datas[3]);
    }

    pub fn set_inner_decoration(&mut self, ideco: InnerDecoration) {
        let datas = Self::unzip(self.datas);
        self.datas = Self::zip_u8(datas[0], datas[1], ideco as u8, datas[3]);
    }

    pub fn set_order(&mut self, order: u8) {
        let datas = Self::unzip(self.datas);
        self.datas = Self::zip_u8(datas[0], datas[1], datas[2], order);
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Geometry2DType {
    Circle = 0,
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
pub enum BorderDecoration {
    None = 0,
    Solid,   // ────
    Dash,    // ----
    DynDash, // ----     (will move)
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InnerDecoration {
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
        assert_eq!(align_of::<Geometry2D>(), 4);
        assert_eq!(size_of::<Geometry2D>(), 32);

        assert_eq!(align_of::<Geometry2DType>(), 1);
        assert_eq!(size_of::<Geometry2DType>(), 1);

        assert_eq!(align_of::<InnerDecoration>(), 1);
        assert_eq!(size_of::<InnerDecoration>(), 1);

        assert_eq!(align_of::<BorderDecoration>(), 1);
        assert_eq!(size_of::<BorderDecoration>(), 1);
    }
}
