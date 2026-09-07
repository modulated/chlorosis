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
//! which does the array access - except RAM/RTC reads and writes, which route
//! through [`Mbc::read_ram`]/[`Mbc::write_ram`] so the controller can serve
//! MBC2's half-width RAM and MBC3's real-time clock. ROM-only, MBC1, MBC2, MBC3
//! (with RTC), and MBC5 are handled; any unrecognised type falls back to MBC1.

use serde::{Deserialize, Serialize};

use crate::{constants::ROM_BANK_SIZE, Byte};

/// External-RAM bank size (8 KB).
const RAM_BANK_SIZE: usize = 0x2000;

/// MBC2's built-in RAM: 512 half-bytes.
const MBC2_RAM_LEN: usize = 512;

/// Master clock ticks in one second of emulated time, the MBC3 RTC's tick base.
const CYCLES_PER_SECOND: u32 = 4_194_304;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Kind {
    None,
    Mbc1,
    Mbc2,
    Mbc3,
    Mbc5,
}

/// The MBC3 real-time clock: five counter registers plus a latched snapshot the
/// CPU reads. Time advances on emulated cycles rather than the wall clock, so it
/// is deterministic and rides along in save states.
#[derive(Debug, Default, Serialize, Deserialize)]
struct Rtc {
    present: bool,
    /// Master ticks accumulated toward the next second.
    cycles: u32,
    seconds: u8,
    minutes: u8,
    hours: u8,
    /// Day counter low 8 bits.
    day_lo: u8,
    /// Day counter high bit (bit 0), halt flag (bit 6), day carry (bit 7).
    day_hi: u8,
    // The snapshot exposed to the CPU between latches.
    l_seconds: u8,
    l_minutes: u8,
    l_hours: u8,
    l_day_lo: u8,
    l_day_hi: u8,
    /// Last value written to the latch register, to catch the 0 -> 1 edge.
    last_latch: u8,
}

impl Rtc {
    /// Advance the clock one master tick, carrying seconds into minutes, hours,
    /// and the 9-bit day counter. The halt flag (day_hi bit 6) freezes it.
    const fn tick(&mut self) {
        if !self.present || self.day_hi & 0x40 != 0 {
            return;
        }
        self.cycles += 1;
        if self.cycles < CYCLES_PER_SECOND {
            return;
        }
        self.cycles = 0;

        self.seconds += 1;
        if self.seconds < 60 {
            return;
        }
        self.seconds = 0;
        self.minutes += 1;
        if self.minutes < 60 {
            return;
        }
        self.minutes = 0;
        self.hours += 1;
        if self.hours < 24 {
            return;
        }
        self.hours = 0;

        let day = (((self.day_hi & 1) as u16) << 8) | self.day_lo as u16;
        let day = day + 1;
        self.day_lo = day as u8;
        // Keep the halt bit; set the day-carry bit on overflow past 511.
        self.day_hi &= 0x40;
        if day > 0x1FF {
            self.day_hi |= 0x80;
        } else {
            self.day_hi |= (day >> 8) as u8 & 1;
        }
    }

    /// Copy the live registers into the latched snapshot the CPU reads.
    const fn latch(&mut self) {
        self.l_seconds = self.seconds;
        self.l_minutes = self.minutes;
        self.l_hours = self.hours;
        self.l_day_lo = self.day_lo;
        self.l_day_hi = self.day_hi;
    }

    const fn read(&self, reg: usize) -> Byte {
        Byte(match reg {
            0x08 => self.l_seconds,
            0x09 => self.l_minutes,
            0x0A => self.l_hours,
            0x0B => self.l_day_lo,
            0x0C => self.l_day_hi,
            _ => 0xFF,
        })
    }

    const fn write(&mut self, reg: usize, val: u8) {
        match reg {
            0x08 => {
                self.seconds = val & 0x3F;
                self.cycles = 0; // writing seconds resets the sub-second divider
            }
            0x09 => self.minutes = val & 0x3F,
            0x0A => self.hours = val & 0x1F,
            0x0B => self.day_lo = val,
            0x0C => self.day_hi = val & 0xC1,
            _ => {}
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mbc {
    kind: Kind,
    rom_banks: usize,
    ram_banks: usize,
    ram_enabled: bool,
    /// Primary ROM bank register: MBC1 low 5 bits, MBC2/MBC3 low bits, MBC5 low 8.
    rom_bank_lo: usize,
    /// Secondary register: MBC1's 2-bit upper/RAM select, MBC5's ROM bit 8.
    rom_bank_hi: usize,
    /// RAM bank select for MBC3/MBC5, or the RTC register select (MBC3).
    ram_bank: usize,
    /// MBC1 advanced banking mode (the `0x6000-0x7FFF` latch).
    advanced_mode: bool,
    rtc: Rtc,
}

impl Mbc {
    /// Build the controller named by the cartridge-type byte at header `0x0147`,
    /// for a ROM of `rom_banks` 16 KB banks and `ram_banks` 8 KB banks.
    #[must_use]
    pub fn new(cartridge_type: u8, rom_banks: usize, ram_banks: usize) -> Self {
        let kind = match cartridge_type {
            0x00 | 0x08 | 0x09 => Kind::None, // ROM (+ optional unbanked RAM)
            0x05 | 0x06 => Kind::Mbc2,
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
            rtc: Rtc {
                // The +TIMER cartridge types carry a clock.
                present: matches!(cartridge_type, 0x0F | 0x10),
                ..Rtc::default()
            },
        }
    }

    /// The external-RAM size a cartridge type needs regardless of the header:
    /// MBC2 has a fixed built-in 512 half-bytes. `None` means use the header.
    #[must_use]
    pub const fn forced_ram_len(cartridge_type: u8) -> Option<usize> {
        match cartridge_type {
            0x05 | 0x06 => Some(MBC2_RAM_LEN),
            _ => None,
        }
    }

    /// Advance the real-time clock (a no-op unless this is an MBC3 with a timer).
    pub const fn tick(&mut self) {
        self.rtc.tick();
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
            // rom_bank_lo is forced to >= 1 on write for MBC1/MBC2/MBC3.
            Kind::Mbc1 => (self.rom_bank_hi << 5) | self.rom_bank_lo,
            Kind::Mbc2 | Kind::Mbc3 => self.rom_bank_lo,
            // MBC5 allows bank 0 in the switchable window.
            Kind::Mbc5 => (self.rom_bank_hi << 8) | self.rom_bank_lo,
        }
    }

    /// Read the external-RAM window `0xA000..=0xBFFF`: an MBC3 RTC register when
    /// one is selected, MBC2's half-width RAM (upper nibble reads as 1), or a
    /// plain RAM byte. Disabled or absent RAM floats to `0xFF`.
    #[must_use]
    pub fn read_ram(&self, addr: u16, eram: &[Byte]) -> Byte {
        if !self.ram_enabled {
            return Byte(0xFF);
        }
        match self.kind {
            Kind::Mbc3 if self.ram_bank >= 0x08 => self.rtc.read(self.ram_bank),
            Kind::Mbc2 => {
                let byte = eram.get(addr as usize & 0x1FF).map_or(0x0F, |b| b.0 & 0x0F);
                Byte(0xF0 | byte)
            }
            _ => self
                .ram_offset(addr)
                .and_then(|o| eram.get(o).copied())
                .unwrap_or(Byte(0xFF)),
        }
    }

    /// Write the external-RAM window: an MBC3 RTC register, MBC2's half-width
    /// RAM (only the low nibble is stored), or a plain RAM byte. Disabled RAM
    /// drops the write.
    pub fn write_ram(&mut self, addr: u16, val: u8, eram: &mut [Byte]) {
        if !self.ram_enabled {
            return;
        }
        match self.kind {
            Kind::Mbc3 if self.ram_bank >= 0x08 => self.rtc.write(self.ram_bank, val),
            Kind::Mbc2 => {
                if let Some(cell) = eram.get_mut(addr as usize & 0x1FF) {
                    *cell = Byte(val & 0x0F);
                }
            }
            _ => {
                if let Some(offset) = self.ram_offset(addr)
                    && let Some(cell) = eram.get_mut(offset)
                {
                    *cell = Byte(val);
                }
            }
        }
    }

    /// Flat external-RAM offset for a `0xA000..=0xBFFF` access, or `None` when
    /// RAM is disabled, absent, or (MBC3) an RTC register is selected. Used by
    /// [`Self::read_ram`]/[`Self::write_ram`] for the plain-RAM case.
    #[must_use]
    const fn ram_offset(&self, addr: u16) -> Option<usize> {
        if !self.ram_enabled || self.ram_banks == 0 {
            return None;
        }
        let bank = match self.kind {
            Kind::None | Kind::Mbc2 => 0,
            Kind::Mbc1 => {
                if self.advanced_mode {
                    self.rom_bank_hi
                } else {
                    0
                }
            }
            Kind::Mbc3 => {
                if self.ram_bank > 0x03 {
                    return None; // RTC register select, handled in read/write_ram
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
            Kind::Mbc2 => {
                // Only 0x0000-0x3FFF is decoded; address bit 8 picks the register.
                if addr < 0x4000 {
                    if addr & 0x0100 == 0 {
                        self.ram_enabled = val & 0x0F == 0x0A;
                    } else {
                        let bank = (val & 0x0F) as usize;
                        self.rom_bank_lo = if bank == 0 { 1 } else { bank };
                    }
                }
            }
            Kind::Mbc3 => match addr {
                0x0000..=0x1FFF => self.ram_enabled = val & 0x0F == 0x0A,
                0x2000..=0x3FFF => {
                    let bank = (val & 0x7F) as usize;
                    self.rom_bank_lo = if bank == 0 { 1 } else { bank };
                }
                0x4000..=0x5FFF => self.ram_bank = (val & 0x0F) as usize,
                _ => {
                    // Writing 0x00 then 0x01 latches the clock into its snapshot.
                    if self.rtc.present && self.rtc.last_latch == 0 && val == 1 {
                        self.rtc.latch();
                    }
                    self.rtc.last_latch = val;
                }
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
    use super::{Kind, Mbc, CYCLES_PER_SECOND};
    use crate::Byte;

    fn mbc1(rom_banks: usize, ram_banks: usize) -> Mbc {
        Mbc::new(0x03, rom_banks, ram_banks)
    }

    #[test]
    fn kind_is_chosen_from_the_cartridge_type() {
        assert_eq!(Mbc::new(0x00, 2, 0).kind, Kind::None);
        assert_eq!(Mbc::new(0x01, 2, 0).kind, Kind::Mbc1);
        assert_eq!(Mbc::new(0x05, 2, 0).kind, Kind::Mbc2);
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

    #[test]
    fn mbc2_uses_four_bit_bank_and_address_bit_eight() {
        let mut m = Mbc::new(0x05, 16, 0);
        // A write with address bit 8 set selects the ROM bank (low 4 bits).
        m.write_control(0x2100, 0x0A);
        assert_eq!(m.rom_offset(0x4000), 0x0A * 0x4000);
        // Bank 0 maps to 1.
        m.write_control(0x2100, 0x00);
        assert_eq!(m.rom_offset(0x4000), 0x4000);
        // A write with address bit 8 clear is the RAM-enable register instead.
        m.write_control(0x2000, 0x0A);
        assert_eq!(m.rom_offset(0x4000), 0x4000, "bit-8-clear write is not a bank");
    }

    #[test]
    fn mbc2_ram_is_four_bit_and_echoes() {
        let mut m = Mbc::new(0x05, 16, 0);
        let mut ram = vec![Byte(0); super::MBC2_RAM_LEN];
        m.write_control(0x0000, 0x0A); // enable RAM

        m.write_ram(0xA000, 0xF7, &mut ram);
        // Only the low nibble is stored; the high nibble reads back as 1s.
        assert_eq!(m.read_ram(0xA000, &ram), Byte(0xF7));
        assert_eq!(ram[0], Byte(0x07), "high nibble discarded");
        // The 512-byte RAM echoes across the whole window.
        assert_eq!(m.read_ram(0xA200, &ram), m.read_ram(0xA000, &ram));
    }

    #[test]
    fn mbc3_rtc_ticks_latches_and_reads() {
        let mut m = Mbc::new(0x10, 4, 1); // MBC3 + TIMER + RAM + BATTERY
        let ram = vec![Byte(0); 0x2000];
        m.write_control(0x0000, 0x0A); // enable RAM/RTC

        // One emulated second of ticks advances the seconds register.
        for _ in 0..CYCLES_PER_SECOND {
            m.tick();
        }
        // Select and latch the seconds register (0x08), then read it.
        m.write_control(0x4000, 0x08);
        m.write_control(0x6000, 0x00);
        m.write_control(0x6000, 0x01); // latch
        assert_eq!(m.read_ram(0xA000, &ram), Byte(1), "one second elapsed");

        // The latched value is frozen until the next latch, even as time runs.
        for _ in 0..CYCLES_PER_SECOND {
            m.tick();
        }
        assert_eq!(m.read_ram(0xA000, &ram), Byte(1), "read is the latched snapshot");
        m.write_control(0x6000, 0x00);
        m.write_control(0x6000, 0x01);
        assert_eq!(m.read_ram(0xA000, &ram), Byte(2), "re-latching sees the new time");
    }

    #[test]
    fn mbc3_rtc_halt_freezes_the_clock() {
        let mut m = Mbc::new(0x10, 4, 1);
        let mut ram = vec![Byte(0); 0x2000];
        m.write_control(0x0000, 0x0A);
        // Select the day-high register and set its halt bit (bit 6). RTC writes
        // ignore the RAM buffer, but the method still takes one.
        m.write_control(0x4000, 0x0C);
        m.write_ram(0xA000, 0x40, &mut ram);

        for _ in 0..CYCLES_PER_SECOND * 3 {
            m.tick();
        }
        // Seconds must still read zero: the halted clock never advanced.
        m.write_control(0x4000, 0x08);
        m.write_control(0x6000, 0x00);
        m.write_control(0x6000, 0x01);
        assert_eq!(m.read_ram(0xA000, &ram), Byte(0), "halted clock did not advance");
    }
}
