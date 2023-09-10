use super::Memory;
use crate::types::{Address, Byte};

// enum BankingMode {
//     RAMBank,
//     ROMBank,
// }

pub struct MBC5 {
    rom_data: Vec<u8>,
    rom_bank: usize,
    ram_bank: usize,
    ram_enabled: bool,
    ram_data: Vec<u8>,
}

const BANK_MAX: usize = 512;
const ROM_BANK_SIZE: usize = 0x4000;
const RAM_BANK_SIZE: usize = 0x2000;

impl Memory for MBC5 {
    fn from_bytes(bytes: Vec<u8>) -> Self {
        if bytes.len() > (BANK_MAX * ROM_BANK_SIZE) {
            panic!("ERROR: Cartrige size greater than expected ROM size");
        }
        Self {
            rom_data: bytes,
            ram_data: vec![0; 0x8000],
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
        }
    }

    fn read(&self, addr: Address) -> Byte {
        match addr.0 {
            0x0000..=0x3FFF => Byte(self.rom_data[addr.0 as usize]),

            0x4000..=0x7FFF => {
                let offset = addr.0 as usize - 0x4000 + (self.rom_bank * ROM_BANK_SIZE);
                Byte(self.rom_data[offset])
            }

            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let offset = addr.0 as usize - 0xA000 + self.ram_bank * RAM_BANK_SIZE;
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
            0x2000..=0x2FFF => {
                self.ram_bank = (self.ram_bank & 0xFF00) + val.0 as usize;
            }
            0x3000..=0x3FFF => {
                self.ram_bank = (self.ram_bank & 0xFF) + (((val.0 as usize) & 0x1) << 8);
            }
            0x4000..=0x5FFF => {
                self.ram_bank = val.0 as usize & 0x0F;
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let offset = self.ram_bank * RAM_BANK_SIZE - 0xA000;
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

    use super::MBC5;

    #[test]
    fn test_ram_bank_set() {
        let mut b = MBC5::from_bytes(vec![]);
        b.write(Address(0x2111), Byte(0x10));
        assert_eq!(b.ram_bank, 0x10);
        b.write(Address(0x3000), Byte(0x11));
        assert_eq!(b.ram_bank, 0x110);
    }
}
