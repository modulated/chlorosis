mod constants;
use super::{Address, Byte};
use constants::*;

#[derive(Debug)]
pub struct MemoryMap {
    cartrige: Vec<Byte>,
    vram: Vec<Byte>,
    wram: Vec<Byte>,
    eram: Vec<Byte>,
    oam: Vec<Byte>,
    hram: Vec<Byte>,
    io: Vec<Byte>,
    interrupt: Byte,
    cartrige_bank: usize,
    vram_bank: usize,
    wram_bank: usize,
}

impl MemoryMap {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn map(&mut self, address: Address) -> &mut Byte {
        match address.0 {
            CARTRIGE_0_START..=CARTRIGE_0_END => &mut self.cartrige[address],
            CARTRIGE_1_START..=CARTRIGE_1_END => {
                &mut self.cartrige[address + Address(CARTRIGE_1_START) * (self.cartrige_bank - 1)]
            }
            VRAM_START..=VRAM_END => {
                &mut self.vram[address + (Address(VRAM_BANK_SIZE as u16) * self.vram_bank)
                    - Address(VRAM_START)]
            }
            ERAM_START..=ERAM_END => &mut self.eram[address - Address(ERAM_START)], // External ram
            WRAM_0_START..=WRAM_0_END => &mut self.wram[address - Address(WRAM_0_START)],
            WRAM_1_START..=WRAM_1_END => {
                &mut self.wram[address + (Address(WRAM_BANK_SIZE as u16) * self.wram_bank)
                    - Address(WRAM_1_START)]
            }
            DEADZONE_0_START..=DEADZONE_0_END => panic!("Prohibited memory access at {address}"),
            OAM_START..=OAM_END => &mut self.oam[address - Address(OAM_START)],
            DEADZONE_1_START..=DEADZONE_1_END => panic!("Prohibited memory access at {address}"),
            IO_START..=IO_END => &mut self.io[address - Address(IO_START)],
            HRAM_START..=HRAM_END => &mut self.hram[address - Address(HRAM_START)],
            INTERRUPT_ENABLE => &mut self.interrupt,
        }
    }

    pub fn read(&mut self, address: Address) -> Byte {
        *self.map(address)
    }

    pub fn write(&mut self, address: Address, value: Byte) {
        *self.map(address) = value;
    }

    pub fn set_cartrige_bank(&mut self, value: usize) {
        self.cartrige_bank = value;
    }

    pub fn load_cartrige(&mut self, mut buf: Vec<u8>) {
        self.cartrige = buf.iter_mut().map(|x| Byte(*x)).collect()
    }

    pub fn dump_cartrige(&mut self) {
        for i in 0x0000..0x8000 {
            let byte = self.read(Address(i));
            if i % 32 == 0 {
                println!();
                print!("{}: ", Address(i));
            }
            if i % 8 == 0 {
                print!("  ");
            }
            print!("{} ", byte);
        }
    }
}

impl Default for MemoryMap {
    fn default() -> Self {
        Self {
            cartrige: vec![],
            vram: vec![Byte(0); VRAM_SIZE],
            wram: vec![Byte(0); WRAM_SIZE],
            eram: vec![Byte(0); ERAM_SIZE],
            oam: vec![Byte(0); OAM_SIZE],
            cartrige_bank: 1,
            vram_bank: 0,
            wram_bank: 1,
            hram: vec![Byte(0); HRAM_SIZE],
            io: vec![Byte(0); IO_SIZE],
            interrupt: Byte(0),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::emu::{
        memory::{MemoryMap, CARTRIGE_BANK_SIZE},
        Address, Byte,
    };

    #[test]
    fn test_memory_map_cartrige_rom_0() {
        let mut mmap = MemoryMap::new();
        mmap.cartrige = vec![Byte(0); CARTRIGE_BANK_SIZE * 3];

        // Start + end of Cartrige ROM bank 00
        mmap.cartrige[0] = Byte(0xFA);
        mmap.cartrige[CARTRIGE_BANK_SIZE - 1] = Byte(0xFF);
        assert_eq!(mmap.read(Address(0x0)), Byte(0xFA));
        assert_eq!(mmap.read(Address(0x3FFF)), Byte(0xFF));
    }
    #[test]
    fn test_memory_map_read_cartrige_rom_1() {
        let mut mmap = MemoryMap::new();
        mmap.cartrige = vec![Byte(0); CARTRIGE_BANK_SIZE * 3];

        // Start + end of Cartrige ROM bank 01
        mmap.cartrige[0x4000] = Byte(0xAA);
        mmap.cartrige[0x4000 + CARTRIGE_BANK_SIZE - 1] = Byte(0xB0);
        assert_eq!(mmap.read(Address(0x4000)), Byte(0xAA));
        assert_eq!(mmap.read(Address(0x7FFF)), Byte(0xB0));
    }
    #[test]
    fn test_memory_map_read_cartrige_rom_2() {
        let mut mmap = MemoryMap::new();
        mmap.cartrige = vec![Byte(0); CARTRIGE_BANK_SIZE * 3];

        // Start + end of Cartrige ROM bank 02
        mmap.cartrige_bank = 2;
        mmap.cartrige[0x8000] = Byte(0xCA);
        mmap.cartrige[0x8000 + CARTRIGE_BANK_SIZE - 1] = Byte(0xBE);
        assert_eq!(mmap.read(Address(0x4000)), Byte(0xCA));
        assert_eq!(mmap.read(Address(0x7FFF)), Byte(0xBE));
    }

    #[test]
    fn test_memory_map_read_vram_0() {
        let mut mmap = MemoryMap::new();

        // Start + end of VRAM bank 00
        mmap.vram[0] = Byte(0xFA);
        mmap.vram[VRAM_BANK_SIZE - 1] = Byte(0xFF);
        assert_eq!(mmap.read(Address(0x8000)), Byte(0xFA));
        assert_eq!(mmap.read(Address(0x9FFF)), Byte(0xFF));
    }

    #[test]
    fn test_memory_map_read_vram_1() {
        let mut mmap = MemoryMap::new();

        // Start + end of VRAM bank 01
        mmap.vram_bank = 1;
        mmap.vram[0x2000] = Byte(0xFB);
        mmap.vram[0x2000 + VRAM_BANK_SIZE - 1] = Byte(0xFE);
        assert_eq!(mmap.read(Address(0x8000)), Byte(0xFB));
        assert_eq!(mmap.read(Address(0x9FFF)), Byte(0xFE));
    }

    // TODO - test external ram + banks

    #[test]
    fn test_memory_map_read_wram_0() {
        let mut mmap = MemoryMap::new();

        // Start + end of WRAM bank 00
        mmap.wram[0] = Byte(0xFA);
        mmap.wram[WRAM_BANK_SIZE - 1] = Byte(0xFF);
        assert_eq!(mmap.read(Address(0xC000)), Byte(0xFA));
        assert_eq!(mmap.read(Address(0xCFFF)), Byte(0xFF));
    }

    #[test]
    fn test_memory_map_read_wram_1() {
        let mut mmap = MemoryMap::new();

        // Start + end of WRAM bank 01
        mmap.wram[0x1000] = Byte(0xFB);
        mmap.wram[0x1000 + WRAM_BANK_SIZE - 1] = Byte(0xFE);
        assert_eq!(mmap.read(Address(0xD000)), Byte(0xFB));
        assert_eq!(mmap.read(Address(0xDFFF)), Byte(0xFE));
    }

    #[test]
    fn test_memory_map_read_wram_2() {
        let mut mmap = MemoryMap::new();

        // Start + end of WRAM bank 02
        mmap.wram_bank = 2;
        mmap.wram[0x2000] = Byte(0xCA);
        mmap.wram[0x2000 + WRAM_BANK_SIZE - 1] = Byte(0xBE);
        assert_eq!(mmap.read(Address(0xD000)), Byte(0xCA));
        assert_eq!(mmap.read(Address(0xDFFF)), Byte(0xBE));
    }

    #[test]
    fn test_memory_map_read_oam() {
        let mut mmap = MemoryMap::new();

        // Start + end of OAM (sprite attribute table)
        mmap.oam[0x0] = Byte(0xCA);
        mmap.oam[OAM_SIZE - 1] = Byte(0xFE);
        assert_eq!(mmap.read(Address(0xFE00)), Byte(0xCA));
        assert_eq!(mmap.read(Address(0xFE9F)), Byte(0xFE));
    }

    #[test]
    fn test_memory_map_read_io_registers() {
        let mut mmap = MemoryMap::new();

        // Start + end of IO Registers
        mmap.io[0x0] = Byte(0xCA);
        mmap.io[IO_SIZE - 1] = Byte(0xFE);
        assert_eq!(mmap.read(Address(0xFF00)), Byte(0xCA));
        assert_eq!(mmap.read(Address(0xFF7F)), Byte(0xFE));
    }

    #[test]
    fn test_memory_map_read_hram() {
        let mut mmap = MemoryMap::new();

        // Start + end of HRAM
        mmap.hram[0x0] = Byte(0xCA);
        mmap.hram[HRAM_SIZE - 1] = Byte(0xFE);
        assert_eq!(mmap.read(Address(0xFF80)), Byte(0xCA));
        assert_eq!(mmap.read(Address(0xFFFE)), Byte(0xFE));
    }

    #[test]
    fn test_memory_map_read_interrupt() {
        let mut mmap = MemoryMap::new();

        mmap.interrupt = Byte(0x10);
        assert_eq!(mmap.read(Address(0xFFFF)), Byte(0x10));
    }

    #[test]
    fn test_memory_map_write_cartrige_rom_0() {
        let mut mmap = MemoryMap::new();
        mmap.cartrige = vec![Byte(0); CARTRIGE_BANK_SIZE * 3];

        // Start + end of Cartrige ROM bank 00
        mmap.write(Address(0x0000), Byte(0xAA));
        mmap.write(Address(0x3FFF), Byte(0xEE));
        assert_eq!(mmap.cartrige[0x0000], Byte(0xAA));
        assert_eq!(mmap.cartrige[CARTRIGE_BANK_SIZE - 1], Byte(0xEE));
    }

    #[test]
    fn test_memory_map_write_cartrige_rom_1() {
        let mut mmap = MemoryMap::new();
        mmap.cartrige = vec![Byte(0); CARTRIGE_BANK_SIZE * 3];

        // Start + end of Cartrige ROM bank 01
        mmap.write(Address(0x4000), Byte(0xAA));
        mmap.write(Address(0x7FFF), Byte(0xEE));
        assert_eq!(mmap.cartrige[0x4000], Byte(0xAA));
        assert_eq!(mmap.cartrige[0x4000 + CARTRIGE_BANK_SIZE - 1], Byte(0xEE));
    }

    #[test]
    fn test_memory_map_write_cartrige_rom_2() {
        let mut mmap = MemoryMap::new();
        mmap.cartrige = vec![Byte(0); CARTRIGE_BANK_SIZE * 3];

        // Start + end of Cartrige ROM bank 02
        mmap.cartrige_bank = 2;
        mmap.write(Address(0x4000), Byte(0xAA));
        mmap.write(Address(0x7FFF), Byte(0xEE));
        assert_eq!(mmap.cartrige[0x8000], Byte(0xAA));
        assert_eq!(mmap.cartrige[0x8000 + CARTRIGE_BANK_SIZE - 1], Byte(0xEE));
    }

    #[test]
    fn test_memory_map_write_vram_0() {
        let mut mmap = MemoryMap::new();

        // Start + end of VRAM bank 00
        mmap.write(Address(0x8000), Byte(0xAA));
        mmap.write(Address(0x9FFF), Byte(0xEE));
        assert_eq!(mmap.vram[0], Byte(0xAA));
        assert_eq!(mmap.vram[VRAM_BANK_SIZE - 1], Byte(0xEE));
    }

    #[test]
    fn test_memory_map_write_vram_1() {
        let mut mmap = MemoryMap::new();

        // Start + end of VRAM bank 01
        mmap.vram_bank = 1;
        mmap.write(Address(0x8000), Byte(0xAA));
        mmap.write(Address(0x9FFF), Byte(0xEE));
        assert_eq!(mmap.vram[0x2000], Byte(0xAA));
        assert_eq!(mmap.vram[0x2000 + VRAM_BANK_SIZE - 1], Byte(0xEE));
    }

    // // TODO - test external ram + banks

    #[test]
    fn test_memory_map_write_wram_0() {
        let mut mmap = MemoryMap::new();

        // Start + end of WRAM bank 00
        mmap.write(Address(0xC000), Byte(0xAA));
        mmap.write(Address(0xCFFF), Byte(0xEE));
        assert_eq!(mmap.wram[0], Byte(0xAA));
        assert_eq!(mmap.wram[WRAM_BANK_SIZE - 1], Byte(0xEE));
    }

    #[test]
    fn test_memory_map_write_wram_1() {
        let mut mmap = MemoryMap::new();

        // Start + end of WRAM bank 01
        mmap.write(Address(0xD000), Byte(0xAA));
        mmap.write(Address(0xDFFF), Byte(0xEE));
        assert_eq!(mmap.wram[mmap.wram_bank * WRAM_BANK_SIZE], Byte(0xAA));
        assert_eq!(
            mmap.wram[(mmap.wram_bank + 1) * WRAM_BANK_SIZE - 1],
            Byte(0xEE)
        );
    }

    #[test]
    fn test_memory_map_write_wram_2() {
        let mut mmap = MemoryMap::new();

        // Start + end of WRAM bank 02
        mmap.wram_bank = 2;
        mmap.write(Address(0xD000), Byte(0xAA));
        mmap.write(Address(0xDFFF), Byte(0xEE));
        assert_eq!(mmap.wram[mmap.wram_bank * WRAM_BANK_SIZE], Byte(0xAA));
        assert_eq!(
            mmap.wram[(mmap.wram_bank + 1) * WRAM_BANK_SIZE - 1],
            Byte(0xEE)
        );
    }

    #[test]
    fn test_memory_map_write_oam() {
        let mut mmap = MemoryMap::new();

        // Start + end of OAM (sprite attribute table)
        mmap.write(Address(0xFE00), Byte(0xAA));
        mmap.write(Address(0xFE9F), Byte(0xEE));
        assert_eq!(mmap.oam[0x0000], Byte(0xAA));
        assert_eq!(mmap.oam[OAM_SIZE - 1], Byte(0xEE));
    }

    #[test]
    fn test_memory_map_write_io_registers() {
        let mut mmap = MemoryMap::new();

        // Start + end of IO Registers
        mmap.write(Address(0xFF00), Byte(0xAA));
        mmap.write(Address(0xFF7F), Byte(0xEE));
        assert_eq!(mmap.io[0x0000], Byte(0xAA));
        assert_eq!(mmap.io[IO_SIZE - 1], Byte(0xEE));
    }

    #[test]
    fn test_memory_map_write_hram() {
        let mut mmap = MemoryMap::new();

        // Start + end of HRAM
        mmap.write(Address(0xFF80), Byte(0xAA));
        mmap.write(Address(0xFFFE), Byte(0xEE));
        assert_eq!(mmap.hram[0x0000], Byte(0xAA));
        assert_eq!(mmap.hram[HRAM_SIZE - 1], Byte(0xEE));
    }

    #[test]
    fn test_memory_map_write_interrupt() {
        let mut mmap = MemoryMap::new();

        mmap.write(Address(0xFFFF), Byte(0x10));
        assert_eq!(mmap.interrupt, Byte(0x10));
    }
}