mod oam;
mod pixel;
mod registers;
mod tile;

use self::{
    pixel::Pixel,
    registers::{StatusMode, TileAddressingMode},
    tile::Tile,
};
use crate::{constants::*, framebuffer::FRAME_LEN, Address, Byte};
use std::collections::VecDeque;

/// Dots (master ticks) per scanline.
const DOTS_PER_LINE: u32 = 456;
/// Dots spent in OAM scan (mode 2) at the start of each visible line.
const OAM_DOTS: u32 = 80;
/// Dots spent in pixel transfer (mode 3). The real duration varies with sprites
/// and scrolling; the minimum is used until the renderer needs otherwise.
const DRAW_DOTS: u32 = 172;
/// First line of VBlank; visible lines are 0..144.
const VBLANK_LINE: u8 = 144;
/// Total lines including the 10 VBlank lines (0..154).
const LINES_PER_FRAME: u8 = 154;

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct PixelProcessor {
    buffer: Option<[u32; FRAME_LEN]>,
    pub vram: [Byte; VRAM_SIZE],
    pub vram_bank: Byte,
    pub oam: [Byte; OAM_SIZE],
    pub bcram: [Byte; 64],
    pub ocram: [Byte; 64],
    line_dot_counter: u32,
    // The pixel-mixing FIFOs the renderer will fill in item 14; unused until
    // then, but kept so the renderer's shape is already carved out.
    #[allow(dead_code)]
    bg_fifo: VecDeque<Pixel>,
    #[allow(dead_code)]
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
            bg_fifo: VecDeque::with_capacity(16),
            obj_fifo: VecDeque::with_capacity(16),
            // Post-boot register state, so the LCD is already on when a
            // cartridge starts (see CentralProcessor::default). LCDC = 0x91:
            // LCD enabled (bit 7), BG tile data at 0x8000 (bit 4), BG enabled
            // (bit 0). BGP = 0xFC is the boot ROM's greyscale palette.
            LCDC: Byte(0x91),
            STAT: Default::default(),
            SCY: Default::default(),
            SCX: Default::default(),
            LY: Default::default(),
            LYC: Default::default(),
            DMA: Default::default(),
            BGP: Byte(0xFC),
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
    /// Take the completed frame, if one is ready.
    ///
    /// The emulation loop pulls frames from here rather than reaching into the
    /// buffer directly, so "a frame is finished" stays a fact the PPU decides.
    pub const fn take_frame(&mut self) -> Option<[u32; FRAME_LEN]> {
        self.buffer.take()
    }

    /// Advance the PPU one dot (one master tick) and return any interrupts this
    /// dot raised.
    ///
    /// This drives the timing state machine only - the position of `LY`, the
    /// STAT mode, LY==LYC coincidence, and the VBlank/STAT interrupts that games
    /// wait on. It does not yet draw pixels: on entering HBlank a scanline would
    /// be rendered, and on entering VBlank the finished frame published, but
    /// that (and the pixel FIFO) is the renderer's job and still to come, so no
    /// frame buffer is produced here.
    pub fn step(&mut self) -> Interrupts {
        // With the LCD off the PPU is idle: LY reads 0, mode reads 0, and
        // nothing is raised. This also keeps it out of `Draw`, where VRAM would
        // be locked.
        if !self.read_lcdc_enabled() {
            self.line_dot_counter = 0;
            self.LY = Byte(0);
            self.write_stat_mode(StatusMode::HBlank);
            self.STAT.write_bit(2, false);
            return Interrupts::empty();
        }

        let mut requested = Interrupts::empty();
        let previous_mode = self.read_stat_mode();

        self.line_dot_counter += 1;
        if self.line_dot_counter >= DOTS_PER_LINE {
            self.line_dot_counter = 0;
            self.LY += 1;
            if self.LY.0 >= LINES_PER_FRAME {
                self.LY = Byte(0);
            }

            // Coincidence is re-evaluated at the start of each line.
            let coincident = self.LY == self.LYC;
            self.STAT.write_bit(2, coincident);
            if coincident && self.STAT.is_bit_set(6) {
                requested |= Interrupts::LCD;
            }
        }

        let mode = self.current_mode();
        if mode != previous_mode {
            self.write_stat_mode(mode);
            requested |= self.on_mode_entry(mode);
        }

        requested
    }

    /// The STAT mode implied by the current `LY` and dot within the line.
    const fn current_mode(&self) -> StatusMode {
        if self.LY.0 >= VBLANK_LINE {
            StatusMode::VBlank
        } else if self.line_dot_counter < OAM_DOTS {
            StatusMode::OAM
        } else if self.line_dot_counter < OAM_DOTS + DRAW_DOTS {
            StatusMode::Draw
        } else {
            StatusMode::HBlank
        }
    }

    /// Side effects of entering a mode: the VBlank interrupt, and the STAT
    /// interrupt for whichever mode-entry sources are enabled.
    fn on_mode_entry(&mut self, mode: StatusMode) -> Interrupts {
        let mut requested = Interrupts::empty();
        match mode {
            StatusMode::OAM => {
                if self.STAT.is_bit_set(5) {
                    requested |= Interrupts::LCD;
                }
            }
            // Item 14: a scanline is rendered into the frame buffer here.
            StatusMode::Draw => {}
            StatusMode::HBlank => {
                if self.STAT.is_bit_set(3) {
                    requested |= Interrupts::LCD;
                }
            }
            StatusMode::VBlank => {
                requested |= Interrupts::VBlank;
                if self.STAT.is_bit_set(4) {
                    requested |= Interrupts::LCD;
                }
                // Item 14: the completed frame is published here.
            }
        }
        requested
    }

    /// Whether the CPU can currently reach VRAM.
    ///
    /// The PPU locks VRAM only during pixel transfer (mode 3, `Draw`); it is
    /// open in HBlank, VBlank, and OAM scan. A locked access is not an error on
    /// real hardware - the read returns `0xFF` and the write is dropped - so it
    /// must never panic here, or every ROM that touches VRAM near a scanline
    /// boundary would take the emulation thread down.
    fn vram_accessible(&self) -> bool {
        !matches!(self.read_stat_mode(), StatusMode::Draw)
    }

    /// Whether the CPU can currently reach OAM. Locked during both OAM scan
    /// (mode 2) and pixel transfer (mode 3); open otherwise.
    fn oam_accessible(&self) -> bool {
        matches!(
            self.read_stat_mode(),
            StatusMode::HBlank | StatusMode::VBlank
        )
    }

    const fn vram_index(&self, address: Address) -> usize {
        address.0 as usize + (VRAM_BANK_SIZE * self.vram_bank.0 as usize) - VRAM_START as usize
    }

    pub fn read_vram(&self, address: Address) -> Byte {
        if self.vram_accessible() {
            self.vram[self.vram_index(address)]
        } else {
            Byte(0xFF)
        }
    }

    pub fn write_vram(&mut self, address: Address, value: Byte) {
        if self.vram_accessible() {
            self.vram[self.vram_index(address)] = value;
        }
    }

    pub fn read_oam(&self, address: Address) -> Byte {
        if self.oam_accessible() {
            self.oam[address.0 as usize - OAM_START as usize]
        } else {
            Byte(0xFF)
        }
    }

    pub fn write_oam(&mut self, address: Address, value: Byte) {
        if self.oam_accessible() {
            self.oam[address.0 as usize - OAM_START as usize] = value;
        }
    }

    // Tiles stored in VRAM, each bank holds 384 tiles (16 bytes each)
    // Split into 3 blocks of 128 tiles
    pub fn get_tile(&self, index: Byte, mode: TileAddressingMode) -> Tile {
        let mut b = [Byte(0); 16];

        match mode {
            TileAddressingMode::Unsigned => {
                for (i, x) in self
                    .vram
                    .iter()
                    .skip(index.0 as usize * TILE_SIZE + self.vram_bank.0 as usize * VRAM_BANK_SIZE)
                    .take(TILE_SIZE)
                    .enumerate()
                {
                    b[i] = *x;
                }
            }
            TileAddressingMode::Signed => {
                if index.0 > 127 {
                    for (i, x) in self
                        .vram
                        .iter()
                        .skip(
                            0x0800
                                + index.0 as usize * TILE_SIZE
                                + self.vram_bank.0 as usize * VRAM_BANK_SIZE,
                        )
                        .take(TILE_SIZE)
                        .enumerate()
                    {
                        b[i] = *x;
                    }
                } else {
                    for (i, x) in self
                        .vram
                        .iter()
                        .skip(
                            index.0 as usize * TILE_SIZE
                                + self.vram_bank.0 as usize * VRAM_BANK_SIZE,
                        )
                        .take(TILE_SIZE)
                        .enumerate()
                    {
                        b[i] = *x;
                    }
                }
            }
        }
        Tile(b)
    }

    fn get_tile_map(&self) -> Vec<Tile> {
        let mut out = Vec::with_capacity(32 * 32);
        let mode = self.read_tile_addressing_mode();
        for i in self.read_background_tile_map_area() {
            let index = self.vram[i as usize]; // TODO: may need to implement bank switch
            out.push(self.get_tile(index, mode))
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::{registers::StatusMode, PixelProcessor};
    use crate::{Address, Byte};

    const VRAM_ADDR: Address = Address(0x8000);
    const OAM_ADDR: Address = Address(0xFE00);

    #[test]
    fn vram_and_oam_are_open_outside_render() {
        let mut ppu = PixelProcessor::default();
        ppu.write_stat_mode(StatusMode::HBlank);

        ppu.write_vram(VRAM_ADDR, Byte(0x42));
        ppu.write_oam(OAM_ADDR, Byte(0x24));

        assert_eq!(ppu.read_vram(VRAM_ADDR), Byte(0x42));
        assert_eq!(ppu.read_oam(OAM_ADDR), Byte(0x24));
    }

    #[test]
    fn vram_locked_during_draw_reads_ff_and_drops_writes() {
        let mut ppu = PixelProcessor::default();
        ppu.write_stat_mode(StatusMode::HBlank);
        ppu.write_vram(VRAM_ADDR, Byte(0x42));

        // A blocked access must not panic - it is routine on real hardware.
        ppu.write_stat_mode(StatusMode::Draw);
        ppu.write_vram(VRAM_ADDR, Byte(0xFF)); // dropped
        assert_eq!(ppu.read_vram(VRAM_ADDR), Byte(0xFF)); // open bus, not the byte

        ppu.write_stat_mode(StatusMode::HBlank);
        assert_eq!(ppu.read_vram(VRAM_ADDR), Byte(0x42)); // write really was dropped
    }

    #[test]
    fn oam_locked_during_scan_and_draw() {
        let mut ppu = PixelProcessor::default();
        ppu.write_stat_mode(StatusMode::HBlank);
        ppu.write_oam(OAM_ADDR, Byte(0x24));

        for blocked in [StatusMode::OAM, StatusMode::Draw] {
            ppu.write_stat_mode(blocked);
            ppu.write_oam(OAM_ADDR, Byte(0xFF)); // dropped
            assert_eq!(ppu.read_oam(OAM_ADDR), Byte(0xFF)); // open bus
        }

        ppu.write_stat_mode(StatusMode::HBlank);
        assert_eq!(ppu.read_oam(OAM_ADDR), Byte(0x24));
    }
}
