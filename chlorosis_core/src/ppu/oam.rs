use crate::types::Byte;

pub struct ObjectAttribute {
    y: Byte,
    x: Byte,
    tile_index: Byte,
    is_occluded: bool,
    xflip: bool,
    yflip: bool,
    // dmp_pallete
    vram_bank: bool, // false = 0, true = 1
    pallete: u8,
}

impl ObjectAttribute {
    pub const fn new(a: Byte, b: Byte, c: Byte, d: Byte) -> Self {
        Self {
            y: a,
            x: b,
            tile_index: c,
            is_occluded: d.is_bit_set(7),
            xflip: d.is_bit_set(6),
            yflip: d.is_bit_set(5),
            vram_bank: d.is_bit_set(3),
            pallete: d.0 & 0b0000_0111,
        }
    }
}
