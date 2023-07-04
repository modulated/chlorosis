use std::ops::RangeInclusive;

use crate::{Address, Byte, PixelProcessor};

impl PixelProcessor {
    pub fn read_io(&self, address: Address) -> Byte {
        match address.0 {
            0xFF40 => self.LCDC,
            0xFF41 => self.STAT,
            0xFF42 => self.SCY,
            0xFF43 => self.SCX,
            0xFF44 => self.LY,
            0xFF45 => self.LYC,
            0xFF46 => self.DMA,
            0xFF47 => self.BGP,
            0xFF48 => self.OBP0,
            0xFF49 => self.OBP1,
            0xFF4A => self.WY,
            0xFF4B => self.WX,
            0xFF4D => self.KEY1,
            0xFF4F => self.vram_bank,
            0xFF55 => self.HDMA5,
            0xFF68 => self.BCPS,
            0xFF69 => self.read_bcram(),
            0xFF6A => self.OCPS,
            0xFF6B => self.read_ocram(),
            0xFF6C => self.OPRI,
            _ => unreachable!("Cannot read IO register {address}"),
        }
    }
    pub fn write_io(&mut self, address: Address, value: Byte) {
        match address.0 {
            0xFF40 => {
                self.LCDC = value & 0b01111111;
                if !value.is_bit_set(7) && self.read_stat_mode() == StatusMode::VBlank {
                    self.lcd_disable();
                } else {
                    self.lcd_enable();
                }
            }
            0xFF41 => {
                self.STAT = value & 0b0111_1100;
            }
            0xFF42 => self.SCY = value,
            0xFF43 => self.SCX = value,
            0xFF45 => self.LYC = value,
            0xFF46 => self.DMA = value,
            0xFF47 => self.BGP = value,
            0xFF48 => self.OBP0 = value,
            0xFF49 => self.OBP1 = value,
            0xFF4A => self.WY = value,
            0xFF4B => self.WX = value,
            0xFF4D => self.KEY1 = value, // MIXED
            0xFF4F => self.vram_bank = value,
            0xFF51 => self.HDMA1 = value,
            0xFF52 => self.HDMA2 = value,
            0xFF53 => self.HDMA3 = value,
            0xFF54 => self.HDMA4 = value,
            0xFF55 => self.HDMA5 = value,
            0xFF68 => self.BCPS = value,
            0xFF69 => self.write_bcpd(value),
            0xFF6A => self.OCPS = value,
            0xFF6B => self.write_ocpd(value),
            0xFF6C => self.OPRI = value,
            _ => unreachable!("Cannot write IO register {address}"),
        }
    }

    pub fn read_stat_mode(&self) -> StatusMode {
        match self.STAT.0 & 0b0000_0011 {
            0b00 => StatusMode::HBlank,
            0b01 => StatusMode::VBlank,
            0b10 => StatusMode::OAM,
            0b11 => StatusMode::Draw,
            _ => unreachable!(),
        }
    }

    pub fn write_stat_mode(&mut self, mode: StatusMode) {
        self.STAT = (self.STAT & 0b1111_1100) + Byte(mode as u8);
    }

    pub const fn read_lcdc_enabled(&self) -> bool {
        self.LCDC.is_bit_set(7)
    }

    fn lcd_disable(&mut self) {
        self.LCDC.write_bit(7, false);
    }

    fn lcd_enable(&mut self) {
        self.LCDC.write_bit(7, true);
    }

    pub const fn read_window_tile_map_area(&self) -> RangeInclusive<u16> {
        if self.LCDC.is_bit_set(6) {
            0x9C00..=0x9FFF
        } else {
            0x9800..=0x9BFF
        }
    }

    pub const fn is_window_enabled(&self) -> bool {
        self.LCDC.is_bit_set(5)
    }

    pub const fn read_tile_addressing_mode(&self) -> TileAddressingMode {
        if self.LCDC.is_bit_set(4) {
            TileAddressingMode::Unsigned
        } else {
            TileAddressingMode::Signed
        }
    }

    pub const fn read_background_tile_map_area(&self) -> RangeInclusive<u16> {
        if self.LCDC.is_bit_set(3) {
            0x9C00..=0x9FFF
        } else {
            0x9800..=0x9BFF
        }
    }

    pub const fn read_obj_size(&self) -> ObjectSize {
        if self.LCDC.is_bit_set(2) {
            ObjectSize::Tall
        } else {
            ObjectSize::Square
        }
    }

    pub const fn is_obj_enabled(&self) -> bool {
        self.LCDC.is_bit_set(1)
    }

    pub const fn is_win_bg_priority(&self) -> bool {
        self.LCDC.is_bit_set(0)
    }

    pub fn write_bcpd(&mut self, value: Byte) {
        if !(self.read_stat_mode() == StatusMode::Draw) {
            self.bcram[self.BCPS.0 as usize & 0x3F] = value;
        }

        if self.BCPS.is_bit_set(7) {
            self.BCPS = Byte((((self.BCPS.0 & 0b0011_1111) + 1) & 0b0011_1111) + 0b1000_0000);
        }
    }

    pub fn write_ocpd(&mut self, value: Byte) {
        if !(self.read_stat_mode() == StatusMode::Draw) {
            self.ocram[self.OCPS.0 as usize & 0x3F] = value;
        }

        if self.BCPS.is_bit_set(7) {
            self.BCPS = Byte((((self.BCPS.0 & 0b0011_1111) + 1) & 0b0011_1111) + 0b1000_0000);
        }
    }

    pub const fn read_bcram(&self) -> Byte {
        self.bcram[self.BCPS.0 as usize]
    }

    pub const fn read_ocram(&self) -> Byte {
        self.ocram[self.OCPS.0 as usize]
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum StatusMode {
    HBlank = 0b00,
    VBlank = 0b01,
    OAM = 0b10,
    Draw = 0b11,
}

pub enum TileAddressingMode {
    Unsigned,
    Signed,
}

pub enum ObjectSize {
    Square,
    Tall,
}

#[cfg(test)]
mod tests {
    use crate::{Byte, PixelProcessor};

    #[test]
    fn test_bcpd_autoincrement() {
        let mut ppu = PixelProcessor {
            BCPS: Byte(0b1011_1111),
            ..Default::default()
        };
        ppu.write_bcpd(Byte(0x1F));
        assert_eq!(ppu.BCPS.0, 0b1000_0000);
    }
}
