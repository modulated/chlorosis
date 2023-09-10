use super::Memory;
use crate::types::{Address, Byte};

enum BankingMode {
    RAMBank,
    ROMBank,
}

pub struct MBC1 {
    rom_data: Vec<u8>,
    rom_bank: usize,
    lower_rom_register: usize,
    upper_rom_register: usize,
    ram_bank: usize,
    ram_enabled: bool,
    mode: BankingMode,
    ram_data: Vec<u8>,
}

const BANK_MAX: usize = 125;

impl Memory for MBC1 {
    fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            rom_data: bytes,
            ram_data: vec![0; 0x8000],
            rom_bank: 1,
            ram_bank: 0,
            upper_rom_register: 0,
            lower_rom_register: 1,
            ram_enabled: false,
            mode: BankingMode::ROMBank,
        }
    }

    fn read(&self, addr: Address) -> Byte {
        match addr.0 {
            0x0000..=0x3FFF => {
                let offset = match self.mode {
                    BankingMode::ROMBank => addr.0 as usize,
                    BankingMode::RAMBank => addr.0 as usize + (self.upper_rom_register << 19),
                };
                Byte(self.rom_data[offset])
            }

            // TODO: mask bits for smaller sized carts
            0x4000..=0x7FFF => {
                let offset = addr.0 as usize
                    + (self.lower_rom_register << 14)
                    + (self.upper_rom_register << 19);
                assert!(offset < self.rom_data.len());
                Byte(self.rom_data[offset])
            }

            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let offset = match self.mode {
                        BankingMode::ROMBank => addr.0 as usize,
                        BankingMode::RAMBank => addr.0 as usize + (self.upper_rom_register << 13),
                    };
                    Byte(self.ram_data[offset])
                } else {
                    Byte(0xFF)
                }
            }

            _ => unreachable!(),
        }
    }

    fn write(&mut self, addr: Address, val: Byte) {
        match addr.0 {
            0x0000..=0x1FFF => self.ram_enabled = (val.0 & 0x0F) == 0x0A,
            0x2000..=0x3FFF => {
                if val == Byte::ZERO {
                    self.rom_bank = 1;
                } else {
                    self.rom_bank = (0b0001_1111 & val.0) as usize;
                    assert!(self.rom_bank <= BANK_MAX);
                }
            }
            0x4000..=0x5FFF => {
                let val = val.0 as usize & 0b0000_0011;
                match self.mode {
                    BankingMode::RAMBank => self.ram_bank = val,
                    BankingMode::ROMBank => {
                        self.rom_bank = (self.rom_bank & 0b0001_1111) + (val << 5);
                        assert!(self.rom_bank <= BANK_MAX);
                    }
                }
            }
            0x6000..=0x7FFF => match val.0 {
                0x00 => self.mode = BankingMode::ROMBank,
                0x01 => self.mode = BankingMode::RAMBank,
                _ => {}
            }, // TODO - RAM Bank 00h can be used during Mode 0, and only ROM Banks 00-1Fh can be used during Mode 1

            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let offset = match self.mode {
                        BankingMode::ROMBank => addr.0 as usize,
                        BankingMode::RAMBank => addr.0 as usize + (self.upper_rom_register << 13),
                    };
                    self.ram_data[offset] = val.0;
                }
            }

            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        mbc::Memory,
        types::{Address, Byte},
    };

    use super::MBC1;

    #[test]
    fn test_ram_enable() {
        let mut mbc = MBC1::from_bytes(vec![]);
        mbc.write(Address(0x0000), Byte(0x1A));
        assert!(mbc.ram_enabled);
        mbc.write(Address(0x1000), Byte(0x11));
        assert!(!mbc.ram_enabled);
    }

    #[test]
    fn test_rom_bank_write() {
        let mut mbc = MBC1::from_bytes(vec![]);
        mbc.write(Address(0x2000), Byte(0xE1));
        assert_eq!(mbc.rom_bank, 1);
        mbc.write(Address(0x2000), Byte(0xE1));
        assert_eq!(mbc.rom_bank, 1);
    }
}
