use crate::types::Byte;

pub struct Tile(pub [Byte; 16]);

pub struct TileAttributes {
    pub priority: bool,
    pub vflip: bool,
    pub hflip: bool,
    vram_bank: bool, // false = 0, true = 1
    palette: u8,
}

impl From<Byte> for TileAttributes {
    fn from(value: Byte) -> Self {
        Self {
            priority: value.is_bit_set(7),
            vflip: value.is_bit_set(6),
            hflip: value.is_bit_set(5),
            vram_bank: value.is_bit_set(3),
            palette: value.0 & 0b0000_0011,
        }
    }
}
