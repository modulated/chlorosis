use super::Memory;
use crate::types::{Address, Byte};

pub struct MBC2 {
    data: Vec<u8>,
    rom_bank: usize,
    ram_enabled: bool,
}

const BANK_MAX: usize = 16;
const BANK_SIZE: usize = 0x4000;

impl Memory for MBC2 {
    fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            data: bytes,
            rom_bank: 1,
            ram_enabled: false,
        }
    }

    fn read(&self, addr: Address) -> Byte {
        match addr.0 {
            0x0000..=0x3FFF => Byte(self.data[addr.0 as usize]),
            0x4000..=0x7FFF => Byte(self.data[addr.0 as usize + BANK_SIZE * self.rom_bank]),

            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    unimplemented!("READ RAM");
                } else {
                    panic!("Attempted to read RAM but NOT ENABLED");
                }
            }

            _ => unreachable!(),
        }
    }

    fn write(&mut self, addr: Address, val: Byte) {
        match addr.0 {
            0x0000..=0x1FFF => {
                let (a, _) = addr.split();
                if a.is_bit_set(0) {
                } else {
                    self.ram_enabled = (val.0 & 0x0F) == 0x0A;
                }
            }
            0x2000..=0x3FFF => {
                let (a, _) = addr.split();
                if !a.is_bit_set(0) {
                } else {
                    self.rom_bank = 0b0000_1111 & val.0 as usize;
                    assert!(self.rom_bank <= BANK_MAX);
                }
            }

            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    unimplemented!("WRITE RAM");
                } else {
                    panic!("Attempted to write RAM but NOT ENABLED");
                }
            }

            _ => unreachable!(),
        }
    }
}
