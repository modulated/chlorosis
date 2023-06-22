// MEMORY ADDRESS POSITION CONSTANTS
pub const BOOT_ROM_START: u16 = 0x0000;
pub const BOOT_ROM_END: u16 = 0x00FF;
pub const CARTRIGE_ENTRY: u16 = 0x0100;
pub const ROM_0_START: u16 = 0x0000;
pub const ROM_0_END: u16 = 0x3FFF;
pub const ROM_1_START: u16 = 0x4000;
pub const ROM_1_END: u16 = 0x7FFF;
pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;
pub const ERAM_START: u16 = 0xA000;
pub const ERAM_END: u16 = 0xBFFF;
pub const WRAM_0_START: u16 = 0xC000;
pub const WRAM_0_END: u16 = 0xCFFF;
pub const WRAM_1_START: u16 = 0xD000;
pub const WRAM_1_END: u16 = 0xDFFF;
pub const DEADZONE_0_START: u16 = 0xE000;
pub const DEADZONE_0_END: u16 = 0xFDFF;
pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;
pub const DEADZONE_1_START: u16 = 0xFEA0;
pub const DEADZONE_1_END: u16 = 0xFEFF;
pub const IO_START: u16 = 0xFF00;
pub const IO_END: u16 = 0xFF7F;
pub const HRAM_START: u16 = 0xFF80;
pub const HRAM_END: u16 = 0xFFFE;
pub const INTERRUPT_ENABLE: u16 = 0xFFFF;

// SIZE CONSTANTS
pub const WRAM_SIZE: usize = 0x8000; // 32 KB
pub const WRAM_BANK_SIZE: usize = 0x1000; // 4 KB
pub const VRAM_SIZE: usize = 0x4000; // 16 KB
pub const VRAM_BANK_SIZE: usize = 0x2000; // 8 KB
pub const ERAM_SIZE: usize = 0x2000; // 8 KB - TODO: ERAM mapper
pub const ROM_BANK_SIZE: usize = 0x4000; // 16 KB
pub const OAM_SIZE: usize = 0xA0; // 160
pub const HRAM_SIZE: usize = 0x7F; // 127
pub const IO_SIZE: usize = 0x80; // 128

// Interrupt flags
bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Interrupts: u8 {
        const VBlank = 0b00000001;
        const LCD = 0b00000010;
        const Timer = 0b00000100;
        const Serial = 0b00001000;
        const Joypad = 0b00010000;
    }
}
