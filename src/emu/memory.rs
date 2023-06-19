use super::{Address, Byte};

const WRAM_SIZE: usize = 0x8000; // 32 KB
const WRAM_BANK_SIZE: usize = 0x1000; // 4 KB
const VRAM_SIZE: usize = 0x4000; // 16 KB
const VRAM_BANK_SIZE: usize = 0x2000; // 8 KB
const ERAM_SIZE: usize = 0x2000; // 8 KB - TODO: ERAM mapper
const CARTRIGE_BANK_SIZE: usize = 0x4000; // 16 KB
const OAM_SIZE: usize = 0xA0; // 160
const HRAM_SIZE: usize = 0x7F; // 127
const IO_SIZE: usize = 0x80; // 128

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
            0x0000..=0x3FFF => &mut self.cartrige[address],
            0x4000..=0x7FFF => {
                &mut self.cartrige[address + Address(0x4000) * (self.cartrige_bank - 1)]
            }
            0x8000..=0x9FFF => {
                &mut self.vram[address + (Address(0x2000) * self.vram_bank) - Address(0x8000)]
            }
            0xA000..=0xBFFF => &mut self.eram[address - Address(0xA000)], // External ram
            0xC000..=0xCFFF => &mut self.wram[address - Address(0xC000)],
            0xD000..=0xDFFF => {
                &mut self.wram[address + (Address(0x1000) * self.wram_bank) - Address(0xD000)]
            }
            0xE000..=0xFDFF => panic!("Prohibited memory access at {address}"),
            0xFE00..=0xFE9F => &mut self.oam[address - Address(0xFE00)],
            0xFEA0..=0xFEFF => panic!("Prohibited memory access at {address}"),
            0xFF00..=0xFF7F => &mut self.io[address - Address(0xFF00)],
            0xFF80..=0xFFFE => &mut self.hram[address - Address(0xFF80)],
            0xFFFF => &mut self.interrupt,
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
    fn test_write() {}
}
