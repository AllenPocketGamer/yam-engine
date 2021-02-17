#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Color {
    Rgb { r: f32, g: f32, b: f32 },
    Rgba { r: f32, g: f32, b: f32, a: f32 },
}

impl Color {
    #[rustfmt::skip]    pub const RED: Self = Self::Rgba {r: 1.0, g: 0.0, b: 0.0, a: 1.0};
    #[rustfmt::skip]    pub const GREEN: Self = Self::Rgba {r: 0.0, g: 1.0, b: 0.0, a: 1.0};
    #[rustfmt::skip]    pub const BLUE: Self = Self::Rgba {r: 0.0, g: 0.0, b: 1.0, a: 1.0};
    #[rustfmt::skip]    pub const WHITE: Self = Self::Rgba {r: 1.0, g: 1.0, b: 1.0, a: 1.0};
    #[rustfmt::skip]    pub const BLACK: Self = Self::Rgba {r: 0.0, g: 0.0, b: 0.0, a: 1.0};

    pub fn to_rgba(&self) -> Self {
        let [r, g, b, a] = self.to_rgba_raw();

        Self::Rgba { r, g, b, a }
    }

    pub fn to_rgba_raw(&self) -> [f32; 4] {
        match self {
            Color::Rgba { r, g, b, a } => [*r, *g, *b, *a],
            Color::Rgb { r, g, b } => [*r, *g, *b, 1.0],
        }
    }
}
