mod pixel;
mod registers;
mod tile;

use self::{pixel::Pixel, registers::StatusMode, tile::Tile};
use crate::{constants::*, Address, Byte};
use std::collections::VecDeque;

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct PixelProcessor {
    pub buffer: Option<[u32; 160 * 144]>,
    pub vram: [Byte; VRAM_SIZE],
    pub vram_bank: Byte,
    pub oam: [Byte; OAM_SIZE],
    pub bcram: [Byte; 64],
    pub ocram: [Byte; 64],
    line_dot_counter: u32,
    frame_dot_counter: u32,
    bg_fifo: VecDeque<Pixel>,
    obj_fifo: VecDeque<Pixel>,
    LCDC: Byte, // LCD control
    STAT: Byte, // PPU state
    SCY: Byte,  // Viewport Y
    SCX: Byte,  // Viewport X
    LY: Byte,   // Current horizontal line being drawn. 0-153. 144 to 153 indicates VBlank
    LYC: Byte,
    DMA: Byte,
    BGP: Byte,  // DMG mode only
    OBP0: Byte, // DMG mode only
    OBP1: Byte, // DMG mode only
    WY: Byte,
    WX: Byte,
    KEY1: Byte, // TODO - this should probably be on CPU
    HDMA1: Byte,
    HDMA2: Byte,
    HDMA3: Byte,
    HDMA4: Byte,
    HDMA5: Byte,
    BCPS: Byte,
    OCPS: Byte,
    OPRI: Byte,
}

impl Default for PixelProcessor {
    fn default() -> Self {
        Self {
            buffer: None,
            vram: [Byte(0); VRAM_SIZE],
            vram_bank: Default::default(),
            oam: [Byte(0); OAM_SIZE],
            bcram: [Byte(0xFF); 64],
            ocram: [Byte(0xFF); 64],
            line_dot_counter: 0,
            frame_dot_counter: 0,
            bg_fifo: VecDeque::with_capacity(16),
            obj_fifo: VecDeque::with_capacity(16),
            LCDC: Default::default(),
            STAT: Default::default(),
            SCY: Default::default(),
            SCX: Default::default(),
            LY: Default::default(),
            LYC: Default::default(),
            DMA: Default::default(),
            BGP: Default::default(),
            OBP0: Default::default(),
            OBP1: Default::default(),
            WY: Default::default(),
            WX: Default::default(),
            KEY1: Default::default(),
            HDMA1: Default::default(),
            HDMA2: Default::default(),
            HDMA3: Default::default(),
            HDMA4: Default::default(),
            HDMA5: Default::default(),
            BCPS: Default::default(),
            OCPS: Default::default(),
            OPRI: Default::default(),
        }
    }
}

impl PixelProcessor {
    pub fn step(&mut self) {
        // Step PPU one dot, runs at 4.194 MHz
        // One frame is 16.74 ms or 70224 dots

        // One line is 456 dots
        // OAM (80 dots) => Draw (172-289 dots) => HBlank (87-204 dots)

        // Check LY=LYC
        self.STAT.write_bit(2, self.LY == self.LYC);
        // TODO: Check for interrupt

        match self.read_stat_mode() {
            StatusMode::HBlank => self.update_line_dot_count(),
            StatusMode::VBlank => {
                if self.frame_dot_counter == 70223 {
                    self.write_stat_mode(StatusMode::OAM);
                    self.frame_dot_counter = 0;
                    self.line_dot_counter = 0;
                    self.LY = Byte(0);
                } else {
                    self.frame_dot_counter += 1;
                    self.update_line_dot_count();
                }
            }
            StatusMode::OAM => self.step_oam(),
            StatusMode::Draw => self.step_draw(),
        }
    }

    fn step_oam(&mut self) {
        if self.LY == self.SCY {
            println!("Draw window");
        }
    }

    fn step_draw(&mut self) {
        self.bg_fifo.clear();
        self.obj_fifo.clear();

        unimplemented!();
    }

    fn update_line_dot_count(&mut self) {
        if self.line_dot_counter == 455 {
            self.line_dot_counter = 0;
            self.LY += 1;
        } else {
            self.line_dot_counter += 1;
        }
    }

    pub fn read_vram(&self, address: Address) -> Byte {
        if !self.read_lcdc_enabled() {
            self.vram[address.0 as usize + (VRAM_BANK_SIZE * self.vram_bank.0 as usize)
                - VRAM_START as usize]
        } else {
            panic!("cannot access VRAM while LCD enabled")
        }
    }

    pub fn write_vram(&mut self, address: Address, value: Byte) {
        if self.read_stat_mode() == StatusMode::VBlank
            || self.read_stat_mode() == StatusMode::HBlank
        {
            self.vram[address.0 as usize + (VRAM_BANK_SIZE * self.vram_bank.0 as usize)
                - VRAM_START as usize] = value;
        } else {
            panic!("Attempted VRAM write during render")
        }
    }

    pub fn read_oam(&self, address: Address) -> Byte {
        if self.read_stat_mode() == StatusMode::VBlank
            || self.read_stat_mode() == StatusMode::HBlank
        {
            self.oam[address.0 as usize - OAM_START as usize]
        } else {
            panic!("Attempted OAM access during render")
        }
    }

    pub fn write_oam(&mut self, address: Address, value: Byte) {
        if self.read_stat_mode() == StatusMode::VBlank
            || self.read_stat_mode() == StatusMode::HBlank
        {
            self.oam[address.0 as usize - OAM_START as usize] = value;
        } else {
            panic!("Attempted OAM write during render time")
        }
    }

    pub fn get_tile(&self, index: Byte) -> Tile {
        let mut b = [Byte(0); 16];
        for (i, x) in self
            .vram
            .iter()
            .skip(index.0 as usize * TILE_SIZE + self.vram_bank.0 as usize * VRAM_BANK_SIZE)
            .take(TILE_SIZE)
            .enumerate()
        {
            b[i] = *x;
        }
        Tile(b)
    }
}
