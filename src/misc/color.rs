pub type Hex = u32;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    #[rustfmt::skip]    pub const RED:          Self = Self::new(255, 0, 0, 255);
    #[rustfmt::skip]    pub const ORANGE:       Self = Self::new(255, 128, 0, 255);
    #[rustfmt::skip]    pub const YELLOW:       Self = Self::new(255, 255, 0, 255);
    #[rustfmt::skip]    pub const CHARTREUSE:   Self = Self::new(128, 255, 0, 255);
    #[rustfmt::skip]    pub const GREEN:        Self = Self::new(0, 255, 0, 255);
    #[rustfmt::skip]    pub const SPRING:       Self = Self::new(0, 255, 128, 255);
    #[rustfmt::skip]    pub const CYAN:         Self = Self::new(0, 255, 255, 255);
    #[rustfmt::skip]    pub const AZURE:        Self = Self::new(0, 128, 255, 255);
    #[rustfmt::skip]    pub const BLUE:         Self = Self::new(0, 0, 255, 255);
    #[rustfmt::skip]    pub const VIOLET:       Self = Self::new(128, 0, 255, 255);
    #[rustfmt::skip]    pub const MAGENTA:      Self = Self::new(255, 0, 255, 255);
    #[rustfmt::skip]    pub const ROSE:         Self = Self::new(255, 0, 128, 255);
    #[rustfmt::skip]    pub const WHITE:        Self = Self::new(255, 255, 255, 255);
    #[rustfmt::skip]    pub const BLACK:        Self = Self::new(0, 0, 0, 255);
    #[rustfmt::skip]    pub const CAMEL:        Self = Self::new(193, 154, 107, 255);
    #[rustfmt::skip]    pub const SOFT_BLACK:   Self = Self::new(14, 17, 17, 255);
    
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn to_hex(&self) -> Hex {
        u32::from_be_bytes([self.r, self.g, self.b, self.a])
    }
    
    pub(crate) fn to_wgpu_color(&self) -> wgpu::Color {
        wgpu::Color {
            r: self.r as f64 / 255.0,
            g: self.g as f64 / 255.0,
            b: self.b as f64 / 255.0,
            a: self.a as f64 / 255.0,
        }
    }
}

unsafe impl bytemuck::Zeroable for Rgba {}
unsafe impl bytemuck::Pod for Rgba {}

#[cfg(test)]
mod test {
    use super::Rgba;
    use std::mem::{size_of, align_of};

    #[test]
    fn check_repr() {
        let size = size_of::<Rgba>();
        let align = align_of::<Rgba>();

        assert_eq!(size, 4);
        assert_eq!(align, 1);
    }
}