#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Pixel {
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
    };
}

impl From<Pixel> for u32 {
    fn from(value: Pixel) -> Self {
        let (r, g, b) = (value.r as Self, value.g as Self, value.b as Self);
        (r << 16) | (g << 8) | b
    }
}
