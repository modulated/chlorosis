use crate::types::Byte;

/// CGB background tile attributes, stored in VRAM bank 1 at the same offset as
/// the tile number in the bank-0 map (`0xFF4F` selects the bank for CPU access,
/// but the renderer reads both banks directly).
pub struct TileAttributes {
    /// BG-to-OAM priority: when set (and LCDC bit 0 is set), this tile's
    /// non-zero pixels draw over sprites.
    pub priority: bool,
    pub vflip: bool,
    pub hflip: bool,
    /// Which VRAM bank holds this tile's pixel data (false = 0, true = 1).
    pub vram_bank: bool,
    /// Background palette number 0-7 into `bcram`.
    pub palette: u8,
}

impl From<Byte> for TileAttributes {
    fn from(value: Byte) -> Self {
        Self {
            priority: value.is_bit_set(7),
            vflip: value.is_bit_set(6),
            hflip: value.is_bit_set(5),
            vram_bank: value.is_bit_set(3),
            palette: value.0 & 0b0000_0111,
        }
    }
}
