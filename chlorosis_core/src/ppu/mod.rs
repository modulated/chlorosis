mod oam;
mod pixel;
mod registers;
mod tile;

use self::{
    pixel::Pixel,
    registers::{ObjectSize, StatusMode, TileAddressingMode},
    tile::Tile,
};
use crate::{
    constants::*,
    framebuffer::{FRAME_LEN, SCREEN_HEIGHT, SCREEN_WIDTH},
    Address, Byte,
};
use std::collections::VecDeque;

/// DMG background shades, darkest last, as `0x00RRGGBB`. The four BGP palette
/// entries index into this; a colour renderer (CGB `bcram`) is a later step.
const SHADES: [u32; 4] = [0x00FF_FFFF, 0x00AA_AAAA, 0x0055_5555, 0x0000_0000];

/// Bytes per tile row (two bitplanes) and rows per tile.
const TILE_ROW_BYTES: usize = 2;
const TILE_HEIGHT: usize = 8;
const TILE_WIDTH: usize = 8;
/// Tiles per row in a 32x32 background map.
const MAP_WIDTH: usize = 32;

/// VRAM byte offset (bank 0) of a background tile's pixel data.
///
/// Unsigned addressing (LCDC bit 4 set) counts tiles up from 0x8000; signed
/// addressing counts from 0x9000 with the tile number taken as `i8`, so numbers
/// 0x80..0xFF address the block just below it.
const fn tile_data_offset(tile_number: u8, unsigned: bool) -> usize {
    if unsigned {
        tile_number as usize * TILE_SIZE
    } else {
        (0x1000 + (tile_number as i8 as isize) * TILE_SIZE as isize) as usize
    }
}

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
    /// Frame handed to the frontend once complete; `None` between frames.
    buffer: Option<[u32; FRAME_LEN]>,
    /// Frame being drawn, one scanline at a time during HBlank. Copied into
    /// `buffer` when the frame finishes at the start of VBlank.
    frame: [u32; FRAME_LEN],
    /// Background colour id (0-3) for each pixel of the line currently being
    /// drawn. Sprites consult it for the BG-over-OBJ priority bit, which the
    /// final `0x00RRGGBB` frame no longer carries.
    bg_line_ids: [u8; SCREEN_WIDTH],
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
            frame: [0; FRAME_LEN],
            bg_line_ids: [0; SCREEN_WIDTH],
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
    /// This drives the timing state machine - the position of `LY`, the STAT
    /// mode, LY==LYC coincidence, and the VBlank/STAT interrupts games wait on -
    /// and, through [`Self::on_mode_entry`], renders each background scanline on
    /// entering HBlank and publishes the frame on entering VBlank. Sprites, the
    /// window, and the cycle-accurate pixel FIFO are still to come.
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
            StatusMode::Draw => {}
            StatusMode::HBlank => {
                // The line just finished pixel transfer; draw it now. Rendering
                // per scanline (rather than once per frame) captures mid-frame
                // scroll changes, which many games rely on.
                self.render_background_line(self.LY.0);
                self.render_sprite_line(self.LY.0);
                if self.STAT.is_bit_set(3) {
                    requested |= Interrupts::LCD;
                }
            }
            StatusMode::VBlank => {
                requested |= Interrupts::VBlank;
                if self.STAT.is_bit_set(4) {
                    requested |= Interrupts::LCD;
                }
                // The frame is complete; hand it to the frontend.
                self.buffer = Some(self.frame);
            }
        }
        requested
    }

    /// Render one background scanline (`ly`, 0..144) into the working frame.
    ///
    /// DMG background only: no window, no sprites, VRAM bank 0, and the BGP
    /// greyscale palette. The line maps into the 256x256 tiled background
    /// through SCX/SCY, wrapping at the edges. With the background disabled
    /// (LCDC bit 0) the line is blanked, as on DMG.
    fn render_background_line(&mut self, ly: u8) {
        let row = ly as usize;
        if row >= SCREEN_HEIGHT {
            return;
        }
        let line_start = row * SCREEN_WIDTH;

        if !self.is_win_bg_priority() {
            let blank = SHADES[0];
            for pixel in &mut self.frame[line_start..line_start + SCREEN_WIDTH] {
                *pixel = blank;
            }
            self.bg_line_ids = [0; SCREEN_WIDTH];
            return;
        }

        let map_base = *self.read_background_tile_map_area().start() as usize;
        let unsigned = matches!(self.read_tile_addressing_mode(), TileAddressingMode::Unsigned);

        let bg_y = ly.wrapping_add(self.SCY.0) as usize;
        let tile_row = bg_y / TILE_HEIGHT;
        let row_in_tile = bg_y % TILE_HEIGHT;

        for screen_x in 0..SCREEN_WIDTH {
            let bg_x = (screen_x as u8).wrapping_add(self.SCX.0) as usize;
            let tile_col = bg_x / TILE_WIDTH;
            let col_in_tile = bg_x % TILE_WIDTH;

            let tile_number = self.vram[map_base + tile_row * MAP_WIDTH + tile_col].0;
            let plane = tile_data_offset(tile_number, unsigned) + row_in_tile * TILE_ROW_BYTES;
            let low = self.vram[plane].0;
            let high = self.vram[plane + 1].0;

            // Pixel 0 of the row is the most significant bit of each plane.
            let bit = 7 - col_in_tile;
            let color_id = (((high >> bit) & 1) << 1) | ((low >> bit) & 1);

            self.bg_line_ids[screen_x] = color_id;
            self.frame[line_start + screen_x] = self.bg_shade(color_id);
        }
    }

    /// Map a 2-bit background colour id through BGP to an `0x00RRGGBB` shade.
    const fn bg_shade(&self, color_id: u8) -> u32 {
        let shade = (self.BGP.0 >> (color_id * 2)) & 0b11;
        SHADES[shade as usize]
    }

    /// Overlay the sprites that intersect scanline `ly` onto the working frame,
    /// on top of the background already drawn there.
    ///
    /// DMG rules: objects are 8x8 or 8x16 (LCDC bit 2); at most 10 per line, in
    /// OAM order; drawn lowest-priority first so lower-X (then lower-OAM-index)
    /// objects land on top. Colour id 0 is transparent, and an object's
    /// priority bit keeps it behind non-zero background pixels.
    fn render_sprite_line(&mut self, ly: u8) {
        if !self.is_obj_enabled() {
            return;
        }
        let height: i16 = match self.read_obj_size() {
            ObjectSize::Tall => 16,
            ObjectSize::Square => 8,
        };
        let ly = ly as i16;

        // Objects intersecting this line, capped at the hardware's 10.
        let mut chosen = [0usize; 10];
        let mut count = 0;
        for i in 0..40 {
            let sprite_y = self.oam[i * 4].0 as i16 - 16;
            if ly >= sprite_y && ly < sprite_y + height {
                chosen[count] = i;
                count += 1;
                if count == 10 {
                    break;
                }
            }
        }
        let chosen = &mut chosen[..count];
        // Priority order is lower X first, ties broken by OAM index.
        chosen.sort_by_key(|&i| (self.oam[i * 4 + 1].0, i));
        // Draw in reverse so the highest-priority object ends up on top.
        for &i in chosen.iter().rev() {
            self.draw_sprite(i, ly, height);
        }
    }

    fn draw_sprite(&mut self, index: usize, ly: i16, height: i16) {
        let sprite_y = self.oam[index * 4].0 as i16 - 16;
        let sprite_x = self.oam[index * 4 + 1].0 as i16 - 8;
        let tile = self.oam[index * 4 + 2].0;
        let attr = self.oam[index * 4 + 3];

        let behind_bg = attr.is_bit_set(7);
        let y_flip = attr.is_bit_set(6);
        let x_flip = attr.is_bit_set(5);
        let palette = if attr.is_bit_set(4) { self.OBP1 } else { self.OBP0 };

        let mut row = (ly - sprite_y) as usize;
        if y_flip {
            row = (height as usize) - 1 - row;
        }
        // In 8x16 mode the low bit of the tile number is ignored; the top tile
        // is even, the bottom odd.
        let tile_number = if height == 16 {
            (tile & 0xFE) | u8::from(row >= 8)
        } else {
            tile
        };
        let plane = tile_data_offset(tile_number, true) + (row % 8) * TILE_ROW_BYTES;
        let low = self.vram[plane].0;
        let high = self.vram[plane + 1].0;

        for col in 0..8usize {
            let x = sprite_x + col as i16;
            if x < 0 || x >= SCREEN_WIDTH as i16 {
                continue;
            }
            let x = x as usize;

            let bit = if x_flip { col } else { 7 - col };
            let color_id = (((high >> bit) & 1) << 1) | ((low >> bit) & 1);
            if color_id == 0 {
                continue; // transparent
            }
            if behind_bg && self.bg_line_ids[x] != 0 {
                continue; // background wins where it is non-zero
            }

            let shade = (palette.0 >> (color_id * 2)) & 0b11;
            self.frame[ly as usize * SCREEN_WIDTH + x] = SHADES[shade as usize];
        }
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
    use super::{registers::StatusMode, PixelProcessor, SHADES};
    use crate::{constants::TILE_SIZE, Address, Byte};

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

    #[test]
    fn background_row_is_rendered_through_the_palette() {
        let mut ppu = PixelProcessor::default(); // LCDC 0x91: LCD+BG on, unsigned
        ppu.BGP = Byte(0xE4); // identity mapping: id N -> shade N

        // Tile 1, row 0: low plane all set, high plane clear -> every pixel id 1.
        ppu.vram[TILE_SIZE] = Byte(0xFF);
        ppu.vram[TILE_SIZE + 1] = Byte(0x00);
        // Background map base is 0x1800 (LCDC bit 3 clear); entry (0,0) -> tile 1.
        ppu.vram[0x1800] = Byte(1);

        ppu.render_background_line(0);

        for (x, pixel) in ppu.frame[0..8].iter().enumerate() {
            assert_eq!(*pixel, SHADES[1], "pixel {x} of tile 1");
        }
        // The next tile-map entry is still 0 -> tile 0 (blank) -> colour id 0.
        assert_eq!(ppu.frame[8], SHADES[0]);
    }

    #[test]
    fn scy_selects_the_tile_row() {
        let mut ppu = PixelProcessor::default();
        ppu.BGP = Byte(0xE4);
        ppu.SCY = Byte(8); // shift the viewport down one tile

        // Tile 1, row 0 -> id 1. With SCY=8, screen line 0 reads background line
        // 8, which is row 0 of the tile in map row 1.
        ppu.vram[TILE_SIZE] = Byte(0xFF);
        ppu.vram[0x1800 + 32] = Byte(1); // map row 1, column 0 -> tile 1

        ppu.render_background_line(0);
        assert_eq!(ppu.frame[0], SHADES[1]);
    }

    #[test]
    fn disabled_background_blanks_the_line() {
        let mut ppu = PixelProcessor::default();
        ppu.LCDC = Byte(0x90); // LCD on, but BG off (bit 0 clear)
        ppu.frame[0] = SHADES[3];

        ppu.render_background_line(0);

        assert_eq!(ppu.frame[0], SHADES[0], "background off blanks to shade 0");
    }

    #[test]
    fn a_full_frame_is_published_at_vblank() {
        let mut ppu = PixelProcessor::default();
        // Run a whole frame; the buffer is handed over on entering VBlank.
        for _ in 0..crate::TICKS_PER_FRAME {
            ppu.step();
        }
        assert!(ppu.take_frame().is_some(), "a frame should be ready");
    }

    /// Place an 8x8 object: OAM entry `slot`, screen position `(x, y)`, using
    /// `tile`, with attribute byte `attr`.
    fn place_sprite(ppu: &mut PixelProcessor, slot: usize, x: u8, y: u8, tile: u8, attr: u8) {
        ppu.oam[slot * 4] = Byte(y + 16);
        ppu.oam[slot * 4 + 1] = Byte(x + 8);
        ppu.oam[slot * 4 + 2] = Byte(tile);
        ppu.oam[slot * 4 + 3] = Byte(attr);
    }

    /// A tile whose every pixel is colour id 3 (both planes set).
    fn solid_tile(ppu: &mut PixelProcessor, tile: u8) {
        for row in 0..8 {
            ppu.vram[tile as usize * TILE_SIZE + row * 2] = Byte(0xFF);
            ppu.vram[tile as usize * TILE_SIZE + row * 2 + 1] = Byte(0xFF);
        }
    }

    #[test]
    fn sprite_is_drawn_over_the_background() {
        let mut ppu = PixelProcessor::default();
        ppu.LCDC = Byte(0x93); // LCD + BG + OBJ on (bits 7,4,1,0)
        ppu.OBP0 = Byte(0xE4); // identity palette
        solid_tile(&mut ppu, 1);
        place_sprite(&mut ppu, 0, 40, 0, 1, 0x00);

        ppu.render_background_line(0);
        ppu.render_sprite_line(0);

        // Background is blank (VRAM zero -> id 0), the 8 sprite pixels are id 3.
        assert_eq!(ppu.frame[40], SHADES[3]);
        assert_eq!(ppu.frame[47], SHADES[3]);
        assert_eq!(ppu.frame[48], SHADES[0], "sprite is 8 pixels wide");
    }

    #[test]
    fn sprite_colour_zero_is_transparent() {
        let mut ppu = PixelProcessor::default();
        ppu.LCDC = Byte(0x93);
        ppu.OBP0 = Byte(0xE4);
        // Tile 1 row 0: only the leftmost pixel is non-zero (id 1).
        ppu.vram[TILE_SIZE] = Byte(0x80);
        place_sprite(&mut ppu, 0, 0, 0, 1, 0x00);
        ppu.frame[1] = SHADES[2]; // pre-existing background under a zero pixel

        ppu.render_sprite_line(0);

        assert_eq!(ppu.frame[0], SHADES[1], "opaque pixel drawn");
        assert_eq!(ppu.frame[1], SHADES[2], "transparent pixel left untouched");
    }

    #[test]
    fn priority_bit_keeps_sprite_behind_non_zero_background() {
        let mut ppu = PixelProcessor::default();
        ppu.LCDC = Byte(0x93);
        ppu.OBP0 = Byte(0xE4);
        solid_tile(&mut ppu, 1);
        place_sprite(&mut ppu, 0, 0, 0, 1, 0x80); // priority bit set

        ppu.bg_line_ids[0] = 2; // background non-zero here
        ppu.bg_line_ids[1] = 0; // background transparent here
        ppu.frame[0] = SHADES[2];
        ppu.render_sprite_line(0);

        assert_eq!(ppu.frame[0], SHADES[2], "hidden behind opaque background");
        assert_eq!(ppu.frame[1], SHADES[3], "shows through where background is 0");
    }

    #[test]
    fn lower_x_sprite_wins() {
        let mut ppu = PixelProcessor::default();
        ppu.LCDC = Byte(0x93);
        ppu.OBP0 = Byte(0xE4); // id 3 -> shade 3
        ppu.OBP1 = Byte(0x24); // id 3 -> shade 0
        solid_tile(&mut ppu, 1);
        // Two overlapping sprites; the lower-X one (slot 1) must win.
        place_sprite(&mut ppu, 0, 4, 0, 1, 0x10); // higher X, OBP1
        place_sprite(&mut ppu, 1, 2, 0, 1, 0x00); // lower X, OBP0

        ppu.render_sprite_line(0);

        // Overlap column 4: slot 1 (OBP0 -> shade 3) is on top.
        assert_eq!(ppu.frame[4], SHADES[3]);
    }
}
