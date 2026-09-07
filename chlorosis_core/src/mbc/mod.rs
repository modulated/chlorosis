//! Cartridge memory-bank controllers (MBCs).
//!
//! The cartridge address space is bank-switched: the CPU sees a fixed 16 KB ROM
//! bank at `0x0000-0x3FFF`, a switchable 16 KB bank at `0x4000-0x7FFF`, and an
//! optional 8 KB external-RAM window at `0xA000-0xBFFF`, but the cartridge
//! behind them can hold megabytes. The MBC chip maps those windows onto the
//! larger image and is programmed by writes into the ROM address range.
//!
//! This owns the banking registers and translates a CPU address into a flat
//! offset; the ROM and external-RAM buffers themselves live in [`crate::Device`],
//! which does the array access. ROM-only, MBC1, MBC3, and MBC5 are handled;
//! other controllers fall back to MBC1 behaviour, and MBC3's real-time clock is
//! not modelled.

use serde::{Deserialize, Serialize};

use crate::constants::ROM_BANK_SIZE;

/// External-RAM bank size (8 KB).
const RAM_BANK_SIZE: usize = 0x2000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Kind {
    None,
    Mbc1,
    Mbc3,
    Mbc5,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mbc {
    kind: Kind,
    rom_banks: usize,
    ram_banks: usize,
    ram_enabled: bool,
    /// Primary ROM bank register: MBC1 low 5 bits, MBC3 7 bits, MBC5 low 8 bits.
    rom_bank_lo: usize,
    /// Secondary register: MBC1's 2-bit upper/RAM select, MBC5's ROM bit 8.
    rom_bank_hi: usize,
    /// RAM bank select for MBC3/MBC5.
    ram_bank: usize,
    /// MBC1 advanced banking mode (the `0x6000-0x7FFF` latch).
    advanced_mode: bool,
}

impl Mbc {
    /// Build the controller named by the cartridge-type byte at header `0x0147`,
    /// for a ROM of `rom_banks` 16 KB banks and `ram_banks` 8 KB banks.
    #[must_use]
    pub fn new(cartridge_type: u8, rom_banks: usize, ram_banks: usize) -> Self {
        let kind = match cartridge_type {
            0x00 | 0x08 | 0x09 => Kind::None, // ROM (+ optional unbanked RAM)
            0x0F..=0x13 => Kind::Mbc3,
            0x19..=0x1E => Kind::Mbc5,
            // 0x01..=0x03 is MBC1; anything unrecognised gets MBC1 as a
            // best-effort default rather than failing to boot.
            _ => Kind::Mbc1,
        };
        Self {
            kind,
            rom_banks: rom_banks.max(1),
            ram_banks,
            ram_enabled: false,
            rom_bank_lo: 1,
            rom_bank_hi: 0,
            ram_bank: 0,
            advanced_mode: false,
        }
    }

    /// Flat ROM offset for a read in `0x0000..=0x7FFF`.
    #[must_use]
    pub const fn rom_offset(&self, addr: u16) -> usize {
        let bank = if addr < 0x4000 {
            // The low window is bank 0, except MBC1 advanced mode maps the high
            // bank bits here too.
            match self.kind {
                Kind::Mbc1 if self.advanced_mode => self.rom_bank_hi << 5,
                _ => 0,
            }
        } else {
            self.switchable_rom_bank()
        };
        (bank % self.rom_banks) * ROM_BANK_SIZE + (addr as usize & (ROM_BANK_SIZE - 1))
    }

    const fn switchable_rom_bank(&self) -> usize {
        match self.kind {
            Kind::None => 1,
            // rom_bank_lo is forced to >= 1 on write for MBC1/MBC3.
            Kind::Mbc1 => (self.rom_bank_hi << 5) | self.rom_bank_lo,
            Kind::Mbc3 => self.rom_bank_lo,
            // MBC5 allows bank 0 in the switchable window.
            Kind::Mbc5 => (self.rom_bank_hi << 8) | self.rom_bank_lo,
        }
    }

    /// Flat external-RAM offset for a `0xA000..=0xBFFF` access, or `None` when
    /// RAM is disabled, absent, or (MBC3) an RTC register is selected.
    #[must_use]
    pub const fn ram_offset(&self, addr: u16) -> Option<usize> {
        if !self.ram_enabled || self.ram_banks == 0 {
            return None;
        }
        let bank = match self.kind {
            Kind::None => 0,
            Kind::Mbc1 => {
                if self.advanced_mode {
                    self.rom_bank_hi
                } else {
                    0
                }
            }
            Kind::Mbc3 => {
                if self.ram_bank > 0x03 {
                    return None; // RTC register select - not modelled
                }
                self.ram_bank
            }
            Kind::Mbc5 => self.ram_bank,
        };
        Some((bank % self.ram_banks) * RAM_BANK_SIZE + (addr as usize - 0xA000))
    }

    /// Handle a CPU write into the ROM range `0x0000..=0x7FFF`, which programs
    /// the banking registers rather than storing anything.
    pub const fn write_control(&mut self, addr: u16, val: u8) {
        match self.kind {
            Kind::None => {}
            Kind::Mbc1 => match addr {
                0x0000..=0x1FFF => self.ram_enabled = val & 0x0F == 0x0A,
                0x2000..=0x3FFF => {
                    let lo = (val & 0x1F) as usize;
                    self.rom_bank_lo = if lo == 0 { 1 } else { lo };
                }
                0x4000..=0x5FFF => self.rom_bank_hi = (val & 0x03) as usize,
                _ => self.advanced_mode = val & 0x01 == 1,
            },
            Kind::Mbc3 => match addr {
                0x0000..=0x1FFF => self.ram_enabled = val & 0x0F == 0x0A,
                0x2000..=0x3FFF => {
                    let bank = (val & 0x7F) as usize;
                    self.rom_bank_lo = if bank == 0 { 1 } else { bank };
                }
                0x4000..=0x5FFF => self.ram_bank = (val & 0x0F) as usize,
                _ => {} // RTC latch - not modelled
            },
            Kind::Mbc5 => match addr {
                0x0000..=0x1FFF => self.ram_enabled = val & 0x0F == 0x0A,
                0x2000..=0x2FFF => self.rom_bank_lo = val as usize,
                0x3000..=0x3FFF => self.rom_bank_hi = (val & 0x01) as usize,
                0x4000..=0x5FFF => self.ram_bank = (val & 0x0F) as usize,
                _ => {}
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Kind, Mbc};

    fn mbc1(rom_banks: usize, ram_banks: usize) -> Mbc {
        Mbc::new(0x03, rom_banks, ram_banks)
    }

    #[test]
    fn kind_is_chosen_from_the_cartridge_type() {
        assert_eq!(Mbc::new(0x00, 2, 0).kind, Kind::None);
        assert_eq!(Mbc::new(0x01, 2, 0).kind, Kind::Mbc1);
        assert_eq!(Mbc::new(0x13, 2, 0).kind, Kind::Mbc3);
        assert_eq!(Mbc::new(0x1B, 2, 0).kind, Kind::Mbc5);
    }

    #[test]
    fn fixed_window_is_bank_zero_and_switchable_follows_the_register() {
        let mut m = mbc1(8, 0);
        assert_eq!(m.rom_offset(0x0000), 0);
        assert_eq!(m.rom_offset(0x3FFF), 0x3FFF);
        // Default switchable bank is 1.
        assert_eq!(m.rom_offset(0x4000), 0x4000);

        m.write_control(0x2000, 5);
        assert_eq!(m.rom_offset(0x4000), 5 * 0x4000);
        assert_eq!(m.rom_offset(0x7FFF), 5 * 0x4000 + 0x3FFF);
    }

    #[test]
    fn switchable_bank_zero_reads_as_bank_one_on_mbc1() {
        let mut m = mbc1(4, 0);
        m.write_control(0x2000, 0); // MBC1 maps bank 0 -> 1 here
        assert_eq!(m.rom_offset(0x4000), 0x4000);
    }

    #[test]
    fn mbc1_upper_bits_extend_the_rom_bank() {
        let mut m = mbc1(128, 0);
        m.write_control(0x2000, 0x01); // low = 1
        m.write_control(0x4000, 0x02); // high 2 bits = 2 -> bank 0x41
        assert_eq!(m.rom_offset(0x4000), 0x41 * 0x4000);
    }

    #[test]
    fn ram_is_gated_by_the_enable_register() {
        let mut m = mbc1(4, 1);
        assert_eq!(m.ram_offset(0xA000), None, "RAM starts disabled");
        m.write_control(0x0000, 0x0A);
        assert_eq!(m.ram_offset(0xA000), Some(0));
        m.write_control(0x0000, 0x00);
        assert_eq!(m.ram_offset(0xA000), None);
    }

    #[test]
    fn mbc5_allows_bank_zero_and_a_ninth_bit() {
        let mut m = Mbc::new(0x19, 512, 0);
        m.write_control(0x2000, 0x00);
        assert_eq!(m.rom_offset(0x4000), 0, "MBC5 keeps bank 0 in the window");
        m.write_control(0x2000, 0x00);
        m.write_control(0x3000, 0x01); // set bit 8 -> bank 0x100
        assert_eq!(m.rom_offset(0x4000), 0x100 * 0x4000);
    }
}
