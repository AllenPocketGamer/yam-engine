use crate::{
    misc::color::{Hex, Rgba},
    nalgebra::Vector2,
};

use std::convert::Into;

pub type Assembly = Vec<Geometry>;

/// (32bytes)Geometry representation.
#[rustfmt::skip]
#[repr(C, packed(4))]
#[derive(Debug, Clone, Copy)]
pub struct Geometry {
    /// 0-7bit   : geometry type
    ///
    /// 8-15bit  : border type
    ///
    /// 16-23bit : inner type
    ///
    /// 24-31bit : order
    datas: u32,

    /// (4bytes)The border color.
    bcolor: Hex,
    /// (4bytes)The inner color.
    icolor: Hex,
    /// (4bytes)Positive represents thickness in `screen space`,
    /// Negative represents thickness in `local space`.
    thickness: f32,

    /// (32bytes)Extras data that control geometry position, rotation and scale.
    extras: [f32; 4],
}

impl Geometry {
    const fn zip(
        gtype: GeometryType,
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

    pub fn new_1d(
        gtype: Geometry1DType,
        bdeco: BorderDecoration,
        bcolor: Rgba,
        border_thickness: BorderThickness,
        order: u8,
        start_point: Vector2<f32>,
        end_point: Vector2<f32>,
    ) -> Self {
        Self {
            datas: Self::zip(gtype.into(), bdeco, InnerDecoration::None, order),
            thickness: border_thickness.to_f32_repr(),
            bcolor: bcolor.to_hex(),
            icolor: Rgba::default().to_hex(),
            extras: [start_point.x, start_point.y, end_point.x, end_point.y],
        }
    }

    pub fn new_2d(
        gtype: Geometry2DType,
        bdeco: BorderDecoration,
        bcolor: Rgba,
        border_thickness: BorderThickness,
        ideco: InnerDecoration,
        icolor: Rgba,
        order: u8,
        position: Vector2<f32>,
        angle: f32,
        size: f32,
    ) -> Self {
        Self {
            datas: Self::zip(gtype.into(), bdeco, ideco, order),
            thickness: border_thickness.to_f32_repr(),
            bcolor: bcolor.to_hex(),
            icolor: icolor.to_hex(),

            extras: [position.x, position.y, angle, size],
        }
    }

    pub fn geometry_type(&self) -> GeometryType {
        unsafe { std::mem::transmute(Self::unzip(self.datas)[0]) }
    }

    pub fn border_decoration(&self) -> BorderDecoration {
        unsafe { std::mem::transmute(Self::unzip(self.datas)[1]) }
    }

    pub fn set_border_decoration(&mut self, bdeco: BorderDecoration) {
        let datas = Self::unzip(self.datas);
        self.datas = Self::zip_u8(datas[0], bdeco as u8, datas[2], datas[3]);
    }

    pub fn border_thickness(&self) -> BorderThickness {
        if self.thickness >= 0.0 {
            BorderThickness::ScreenSpace(self.thickness)
        } else {
            BorderThickness::LocalSpace(f32::abs(self.thickness))
        }
    }

    pub fn set_border_thickness(&mut self, bt: BorderThickness) {
        self.thickness = bt.to_f32_repr()
    }

    pub fn inner_decoration(&self) -> InnerDecoration {
        unsafe { std::mem::transmute(Self::unzip(self.datas)[2]) }
    }

    pub fn set_inner_decoration(&mut self, ideco: InnerDecoration) {
        let datas = Self::unzip(self.datas);
        self.datas = Self::zip_u8(datas[0], datas[1], ideco as u8, datas[3]);
    }

    pub const fn order(&self) -> u8 {
        Self::unzip(self.datas)[3]
    }

    pub fn set_order(&mut self, order: u8) {
        let datas = Self::unzip(self.datas);
        self.datas = Self::zip_u8(datas[0], datas[1], datas[2], order);
    }

    pub const fn border_color_hex(&self) -> Hex {
        self.bcolor
    }

    pub fn set_border_color_hex(&mut self, hex: Hex) {
        self.bcolor = hex
    }

    pub const fn inner_color_hex(&self) -> Hex {
        self.icolor
    }

    pub fn set_inner_color_hex(&mut self, hex: Hex) {
        self.icolor = hex
    }

    pub const fn border_color_rgba(&self) -> Rgba {
        Rgba::from_hex(self.bcolor)
    }

    pub fn set_border_color_rgba(&mut self, rgba: &Rgba) {
        self.bcolor = rgba.to_hex()
    }

    pub const fn inner_color_rgba(&self) -> Rgba {
        Rgba::from_hex(self.icolor)
    }

    pub fn set_inner_color_rgba(&mut self, rgba: &Rgba) {
        self.icolor = rgba.to_hex()
    }

    pub fn position_uncheck(&self) -> Vector2<f32> {
        Vector2::new(self.extras[0], self.extras[1])
    }

    pub fn set_position_uncheck(&mut self, position: Vector2<f32>) {
        self.extras[0] = position.x;
        self.extras[1] = position.y;
    }

    pub const fn angle_uncheck(&self) -> f32 {
        self.extras[2]
    }

    pub fn set_angle_uncheck(&mut self, angle: f32) {
        self.extras[2] = angle
    }

    pub const fn size_uncheck(&self) -> f32 {
        self.extras[3]
    }

    pub fn set_size_uncheck(&mut self, size: f32) {
        self.extras[3] = size
    }

    pub fn start_point_uncheck(&self) -> Vector2<f32> {
        Vector2::new(self.extras[0], self.extras[1])
    }

    pub fn set_start_point_uncheck(&mut self, start_point: Vector2<f32>) {
        self.extras[0] = start_point.x;
        self.extras[1] = start_point.y;
    }

    pub fn end_point_uncheck(&self) -> Vector2<f32> {
        Vector2::new(self.extras[2], self.extras[3])
    }

    pub fn set_end_point_uncheck(&mut self, end_point: Vector2<f32>) {
        self.extras[2] = end_point.x;
        self.extras[3] = end_point.y;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorderThickness {
    /// Thickness in `local space`.
    LocalSpace(f32),
    /// Thickness in `screen space`.
    ScreenSpace(f32),
}

impl BorderThickness {
    fn to_f32_repr(self) -> f32 {
        match self {
            BorderThickness::LocalSpace(th) => -th,
            BorderThickness::ScreenSpace(th) => th,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryType {
    Circle = 0,
    ETriangle = 1,
    Square = 2,
    Pentagon = 3,
    Hexagon = 4,
    Octogon = 5,
    Hexagram = 6,
    StarFive = 7,
    Heart = 8,

    Line = 20,
    Ray = 21,
    Segment = 22,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Geometry1DType {
    Line = 0,
    Ray = 1,
    Segment = 2,
}

impl Into<GeometryType> for Geometry1DType {
    fn into(self) -> GeometryType {
        match self {
            Self::Line => GeometryType::Line,
            Self::Ray => GeometryType::Ray,
            Self::Segment => GeometryType::Segment,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Geometry2DType {
    Circle = 0,
    ETriangle = 1,
    Square = 2,
    Pentagon = 3,
    Hexagon = 4,
    Octogon = 5,
    Hexagram = 6,
    StarFive = 7,
    Heart = 8,
}

impl Into<GeometryType> for Geometry2DType {
    fn into(self) -> GeometryType {
        match self {
            Self::Circle => GeometryType::Circle,
            Self::ETriangle => GeometryType::ETriangle,
            Self::Square => GeometryType::Square,
            Self::Pentagon => GeometryType::Pentagon,
            Self::Hexagon => GeometryType::Hexagon,
            Self::Octogon => GeometryType::Octogon,
            Self::Hexagram => GeometryType::Hexagram,
            Self::StarFive => GeometryType::StarFive,
            Self::Heart => GeometryType::Heart,
        }
    }
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
        assert_eq!(align_of::<Geometry>(), 4);
        assert_eq!(size_of::<Geometry>(), 32);

        assert_eq!(align_of::<GeometryType>(), 1);
        assert_eq!(size_of::<GeometryType>(), 1);

        assert_eq!(align_of::<InnerDecoration>(), 1);
        assert_eq!(size_of::<InnerDecoration>(), 1);

        assert_eq!(align_of::<BorderDecoration>(), 1);
        assert_eq!(size_of::<BorderDecoration>(), 1);
    }
}
