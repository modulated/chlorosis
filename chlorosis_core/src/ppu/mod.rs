mod pixel;
mod registers;
mod tile;

use self::{
    pixel::Pixel,
    registers::{ObjectSize, StatusMode, TileAddressingMode},
    tile::TileAttributes,
};
use crate::{
    constants::*,
    framebuffer::{FRAME_LEN, SCREEN_HEIGHT, SCREEN_WIDTH},
    Address, Byte,
};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
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
// Defaults for the render buffers a save state skips (arrays larger than 32
// have no `Default`, so serde needs these by name).
const fn no_frame() -> Option<Box<[u32; FRAME_LEN]>> {
    None
}
fn blank_frame_buffer() -> Box<[u32; FRAME_LEN]> {
    Box::new([0; FRAME_LEN])
}
const fn blank_line_ids() -> [u8; SCREEN_WIDTH] {
    [0; SCREEN_WIDTH]
}
const fn blank_line_priority() -> [bool; SCREEN_WIDTH] {
    [false; SCREEN_WIDTH]
}

const fn tile_data_offset(tile_number: u8, unsigned: bool) -> usize {
    if unsigned {
        tile_number as usize * TILE_SIZE
    } else {
        (0x1000 + (tile_number as i8 as isize) * TILE_SIZE as isize) as usize
    }
}

/// Convert one CGB palette entry to `0x00RRGGBB`. `ram` is `bcram` (background)
/// or `ocram` (objects); each palette is eight bytes holding four little-endian
/// BGR555 colours, so entry `color_id` of `palette` starts at `palette*8 +
/// color_id*2`. Each 5-bit channel is expanded to 8 bits.
const fn cgb_color(ram: &[Byte; 64], palette: u8, color_id: u8) -> u32 {
    let base = palette as usize * 8 + color_id as usize * 2;
    let rgb555 = ram[base].0 as u16 | ((ram[base + 1].0 as u16) << 8);
    let r = (rgb555 & 0x1F) as u32;
    let g = ((rgb555 >> 5) & 0x1F) as u32;
    let b = ((rgb555 >> 10) & 0x1F) as u32;
    // 5-bit -> 8-bit: shift up and replicate the top bits into the low ones.
    let r = (r << 3) | (r >> 2);
    let g = (g << 3) | (g >> 2);
    let b = (b << 3) | (b >> 2);
    (r << 16) | (g << 8) | b
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

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct PixelProcessor {
    // The frame buffers and per-line scratch are transient render output, not
    // machine state: they are rebuilt from VRAM/OAM on the next scanline, so a
    // save state skips them and reconstructs blanks on load.
    /// Frame handed to the frontend once complete; `None` between frames. Boxed
    /// so the ~90 KB buffer lives on the heap rather than bloating the struct
    /// (which is moved wholesale on reset and on loading a save state).
    #[serde(skip, default = "no_frame")]
    buffer: Option<Box<[u32; FRAME_LEN]>>,
    /// Frame being drawn, one scanline at a time during HBlank. Copied into
    /// `buffer` when the frame finishes at the start of VBlank.
    #[serde(skip, default = "blank_frame_buffer")]
    frame: Box<[u32; FRAME_LEN]>,
    /// Background colour id (0-3) for each pixel of the line currently being
    /// drawn. Sprites consult it for the BG-over-OBJ priority bit, which the
    /// final `0x00RRGGBB` frame no longer carries.
    #[serde(skip, default = "blank_line_ids")]
    bg_line_ids: [u8; SCREEN_WIDTH],
    /// Whether this tile's background pixel has BG-to-OAM priority set (CGB tile
    /// attribute bit 7). Sprites yield to it where the pixel is non-zero.
    #[serde(skip, default = "blank_line_priority")]
    bg_line_priority: [bool; SCREEN_WIDTH],
    /// Colour rendering (CGB palettes + tile attributes) versus DMG greyscale.
    /// Set from the cartridge's CGB flag when a ROM loads.
    cgb_mode: bool,
    /// The window's own line counter. Unlike the background, the window advances
    /// only on scanlines where it is actually drawn, so it is tracked separately
    /// from `LY` and reset each frame.
    window_line: u8,
    #[serde(with = "BigArray")]
    pub vram: [Byte; VRAM_SIZE],
    pub vram_bank: Byte,
    #[serde(with = "BigArray")]
    pub oam: [Byte; OAM_SIZE],
    #[serde(with = "BigArray")]
    pub bcram: [Byte; 64],
    #[serde(with = "BigArray")]
    pub ocram: [Byte; 64],
    line_dot_counter: u32,
    // The pixel-mixing FIFOs the renderer will fill in item 14; unused until
    // then, but kept so the renderer's shape is already carved out.
    #[allow(dead_code)]
    #[serde(skip)]
    bg_fifo: VecDeque<Pixel>,
    #[allow(dead_code)]
    #[serde(skip)]
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
            frame: Box::new([0; FRAME_LEN]),
            bg_line_ids: [0; SCREEN_WIDTH],
            bg_line_priority: [false; SCREEN_WIDTH],
            cgb_mode: false,
            window_line: 0,
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
    pub const fn take_frame(&mut self) -> Option<Box<[u32; FRAME_LEN]>> {
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
                self.render_window_line(self.LY.0);
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
                // The frame is complete; hand it to the frontend. The window's
                // line counter restarts for the next frame.
                self.buffer = Some(self.frame.clone());
                self.window_line = 0;
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

        // LCDC bit 0 blanks the background on DMG. On CGB it instead only drops
        // the background's priority over sprites, so the tiles still draw.
        if !self.cgb_mode && !self.is_win_bg_priority() {
            let blank = SHADES[0];
            for pixel in &mut self.frame[line_start..line_start + SCREEN_WIDTH] {
                *pixel = blank;
            }
            self.bg_line_ids = [0; SCREEN_WIDTH];
            self.bg_line_priority = [false; SCREEN_WIDTH];
            return;
        }

        let map_base = *self.read_background_tile_map_area().start() as usize;
        let unsigned = matches!(self.read_tile_addressing_mode(), TileAddressingMode::Unsigned);

        let bg_y = ly.wrapping_add(self.SCY.0) as usize;

        for screen_x in 0..SCREEN_WIDTH {
            let bg_x = (screen_x as u8).wrapping_add(self.SCX.0) as usize;
            let (color_id, attr) = self.tile_pixel(map_base, bg_x, bg_y, unsigned);

            self.bg_line_ids[screen_x] = color_id;
            self.bg_line_priority[screen_x] = attr.priority;
            self.frame[line_start + screen_x] = self.bg_color(color_id, &attr);
        }
    }

    /// Render the window layer over the scanline `ly`, where it is enabled and
    /// visible. The window is a second tile map (LCDC bit 6) drawn at a fixed
    /// screen position (`WX-7`, `WY`) with its own line counter, so it does not
    /// scroll with the background. It counts as background for sprite priority.
    fn render_window_line(&mut self, ly: u8) {
        if !self.is_window_enabled() {
            return;
        }
        // On DMG the window needs the BG/window master enable (LCDC bit 0); on
        // CGB that bit is only a priority control, so the window always draws.
        if !self.cgb_mode && !self.is_win_bg_priority() {
            return;
        }
        let row = ly as usize;
        if row >= SCREEN_HEIGHT || ly < self.WY.0 {
            return;
        }
        // The window's left edge is WX-7; a WX past the screen hides it entirely.
        let left = self.WX.0 as i16 - 7;
        if left >= SCREEN_WIDTH as i16 {
            return;
        }

        let map_base = *self.read_window_tile_map_area().start() as usize - VRAM_START as usize;
        let unsigned = matches!(self.read_tile_addressing_mode(), TileAddressingMode::Unsigned);
        let win_y = self.window_line as usize;
        let line_start = row * SCREEN_WIDTH;

        for screen_x in 0..SCREEN_WIDTH {
            if (screen_x as i16) < left {
                continue;
            }
            let win_x = (screen_x as i16 - left) as usize;
            let (color_id, attr) = self.tile_pixel(map_base, win_x, win_y, unsigned);

            self.bg_line_ids[screen_x] = color_id;
            self.bg_line_priority[screen_x] = attr.priority;
            self.frame[line_start + screen_x] = self.bg_color(color_id, &attr);
        }
        // The counter only advances on lines the window was actually drawn.
        self.window_line = self.window_line.wrapping_add(1);
    }

    /// Fetch a background/window pixel: its 2-bit colour id and CGB attributes.
    /// `map_base` is the VRAM-relative tile-map offset; `(px, py)` is the pixel
    /// within the 256x256 map. On DMG the attributes are all-zero (palette 0,
    /// bank 0, no flip), which reduces this to the plain background fetch.
    fn tile_pixel(
        &self,
        map_base: usize,
        px: usize,
        py: usize,
        unsigned: bool,
    ) -> (u8, TileAttributes) {
        let tile_row = py / TILE_HEIGHT;
        let tile_col = px / TILE_WIDTH;
        let map_index = map_base + tile_row * MAP_WIDTH + tile_col;
        let tile_number = self.vram[map_index].0;

        // CGB tile attributes live at the same offset in VRAM bank 1.
        let attr = if self.cgb_mode {
            TileAttributes::from(self.vram[VRAM_BANK_SIZE + map_index])
        } else {
            TileAttributes::from(Byte(0))
        };

        let row_in_tile = if attr.vflip {
            TILE_HEIGHT - 1 - (py % TILE_HEIGHT)
        } else {
            py % TILE_HEIGHT
        };
        let col_in_tile = px % TILE_WIDTH;
        let bit = if attr.hflip { col_in_tile } else { 7 - col_in_tile };

        let bank_offset = if attr.vram_bank { VRAM_BANK_SIZE } else { 0 };
        let plane =
            bank_offset + tile_data_offset(tile_number, unsigned) + row_in_tile * TILE_ROW_BYTES;
        let low = self.vram[plane].0;
        let high = self.vram[plane + 1].0;
        let color_id = (((high >> bit) & 1) << 1) | ((low >> bit) & 1);
        (color_id, attr)
    }

    /// Map a background/window colour id to `0x00RRGGBB`: through `bcram` in CGB
    /// mode, or the BGP greyscale ramp on DMG.
    const fn bg_color(&self, color_id: u8, attr: &TileAttributes) -> u32 {
        if self.cgb_mode {
            cgb_color(&self.bcram, attr.palette, color_id)
        } else {
            self.bg_shade(color_id)
        }
    }

    /// Map a 2-bit background colour id through BGP to an `0x00RRGGBB` shade.
    const fn bg_shade(&self, color_id: u8) -> u32 {
        let shade = (self.BGP.0 >> (color_id * 2)) & 0b11;
        SHADES[shade as usize]
    }

    /// Select the colour renderer (CGB palettes) or the DMG greyscale path.
    pub const fn set_cgb_mode(&mut self, on: bool) {
        self.cgb_mode = on;
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
        // DMG orders objects by X (ties broken by OAM index); CGB orders purely
        // by OAM index, so lower-index objects always win. The scan already
        // collected them in OAM order, so CGB just keeps that.
        if !self.cgb_mode {
            chosen.sort_by_key(|&i| (self.oam[i * 4 + 1].0, i));
        }
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
        // DMG picks one of two palettes with bit 4; CGB uses bit 3 to pick the
        // VRAM bank of the tile data and bits 0-2 to pick an OBJ palette.
        let dmg_palette = if attr.is_bit_set(4) { self.OBP1 } else { self.OBP0 };
        let cgb_palette = attr.0 & 0b0000_0111;
        let bank_offset = if self.cgb_mode && attr.is_bit_set(3) {
            VRAM_BANK_SIZE
        } else {
            0
        };
        // On CGB, LCDC bit 0 is a master switch: cleared, objects always win.
        let bg_has_priority = !self.cgb_mode || self.is_win_bg_priority();

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
        let plane = bank_offset + tile_data_offset(tile_number, true) + (row % 8) * TILE_ROW_BYTES;
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
            // The object yields to non-zero background where the background has
            // priority - set either by this object's own OAM bit 7 or, on CGB,
            // by the background tile's own priority attribute.
            let bg_wins = bg_has_priority
                && self.bg_line_ids[x] != 0
                && (behind_bg || self.bg_line_priority[x]);
            if bg_wins {
                continue;
            }

            self.frame[ly as usize * SCREEN_WIDTH + x] = if self.cgb_mode {
                cgb_color(&self.ocram, cgb_palette, color_id)
            } else {
                let shade = (dmg_palette.0 >> (color_id * 2)) & 0b11;
                SHADES[shade as usize]
            };
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

    /// CGB VRAM DMA parameters: `(source, destination, length)`. Source is
    /// `HDMA1:HDMA2` aligned to 16 bytes; destination is `HDMA3:HDMA4`, an offset
    /// into VRAM (`0x8000`); length is `((HDMA5 & 0x7F) + 1) * 16`.
    pub const fn hdma_params(&self) -> (u16, u16, usize) {
        let source = ((self.HDMA1.0 as u16) << 8 | self.HDMA2.0 as u16) & 0xFFF0;
        let dest = (((self.HDMA3.0 as u16) << 8 | self.HDMA4.0 as u16) & 0x1FF0) | VRAM_START;
        let length = ((self.HDMA5.0 & 0x7F) as usize + 1) * 16;
        (source, dest, length)
    }

    /// Write one byte into VRAM (current bank) during a DMA, bypassing the
    /// mode-based access guard that applies to CPU writes.
    pub fn dma_write_vram(&mut self, address: Address, value: Byte) {
        let index = self.vram_index(address);
        if let Some(cell) = self.vram.get_mut(index) {
            *cell = value;
        }
    }

    /// Mark the VRAM DMA finished: `HDMA5` reads `0xFF` (bit 7 set = no active
    /// transfer, length 0x7F).
    pub const fn hdma_finish(&mut self) {
        self.HDMA5 = Byte(0xFF);
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

}

#[cfg(test)]
mod tests {
    use super::{registers::StatusMode, PixelProcessor, SHADES};
    use crate::{
        constants::{TILE_SIZE, VRAM_BANK_SIZE},
        Address, Byte,
    };

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
        // Default LCDC 0x91: LCD+BG on, unsigned addressing.
        let mut ppu = PixelProcessor {
            BGP: Byte(0xE4), // identity mapping: id N -> shade N
            ..Default::default()
        };

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
    fn cgb_background_uses_palette_ram_and_tile_attributes() {
        let mut ppu = PixelProcessor::default();
        ppu.set_cgb_mode(true);

        // Tile 1, row 0: low plane all set, high clear -> every pixel colour id 1.
        ppu.vram[TILE_SIZE] = Byte(0xFF);
        ppu.vram[TILE_SIZE + 1] = Byte(0x00);
        // Map entry (0,0) -> tile 1 (bank 0); its attribute lives at the same
        // offset in bank 1. Palette 2, no flip, tile data in bank 0.
        ppu.vram[0x1800] = Byte(1);
        ppu.vram[VRAM_BANK_SIZE + 0x1800] = Byte(0b0000_0010);

        // Background palette 2, colour 1: pure red in BGR555 (r=0x1F) at byte
        // offset 2*8 + 1*2 = 18, little-endian.
        ppu.bcram[18] = Byte(0x1F);
        ppu.bcram[19] = Byte(0x00);

        ppu.render_background_line(0);

        // 5-bit 0x1F expands to 8-bit 0xFF, so the pixels are opaque red.
        for (x, pixel) in ppu.frame[0..8].iter().enumerate() {
            assert_eq!(*pixel, 0x00FF_0000, "pixel {x} should be palette-2 red");
        }
    }

    #[test]
    fn cgb_background_honours_horizontal_flip() {
        let mut ppu = PixelProcessor::default();
        ppu.set_cgb_mode(true);

        // Tile 1, row 0: only the leftmost pixel (MSB) is colour id 1.
        ppu.vram[TILE_SIZE] = Byte(0b1000_0000);
        ppu.vram[TILE_SIZE + 1] = Byte(0x00);
        ppu.vram[0x1800] = Byte(1);
        // Attribute: palette 0, horizontal flip (bit 5).
        ppu.vram[VRAM_BANK_SIZE + 0x1800] = Byte(0b0010_0000);

        // Palette 0 colour 0 (byte 0) opaque black, colour 1 (byte 2) red.
        ppu.bcram[0] = Byte(0x00);
        ppu.bcram[1] = Byte(0x00);
        ppu.bcram[2] = Byte(0x1F);
        ppu.bcram[3] = Byte(0x00);

        ppu.render_background_line(0);

        // Flipped, the set pixel lands at the right edge of the tile (x = 7).
        assert_eq!(ppu.frame[7], 0x00FF_0000, "flipped set pixel at x=7");
        assert_eq!(ppu.frame[0], 0x0000_0000, "x=0 is now colour 0");
    }

    #[test]
    fn scy_selects_the_tile_row() {
        let mut ppu = PixelProcessor {
            BGP: Byte(0xE4),
            SCY: Byte(8), // shift the viewport down one tile
            ..Default::default()
        };

        // Tile 1, row 0 -> id 1. With SCY=8, screen line 0 reads background line
        // 8, which is row 0 of the tile in map row 1.
        ppu.vram[TILE_SIZE] = Byte(0xFF);
        ppu.vram[0x1800 + 32] = Byte(1); // map row 1, column 0 -> tile 1

        ppu.render_background_line(0);
        assert_eq!(ppu.frame[0], SHADES[1]);
    }

    #[test]
    fn disabled_background_blanks_the_line() {
        let mut ppu = PixelProcessor {
            LCDC: Byte(0x90), // LCD on, but BG off (bit 0 clear)
            ..Default::default()
        };
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
        let mut ppu = PixelProcessor {
            LCDC: Byte(0x93), // LCD + BG + OBJ on (bits 7,4,1,0)
            OBP0: Byte(0xE4), // identity palette
            ..Default::default()
        };
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
        let mut ppu = PixelProcessor {
            LCDC: Byte(0x93),
            OBP0: Byte(0xE4),
            ..Default::default()
        };
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
        let mut ppu = PixelProcessor {
            LCDC: Byte(0x93),
            OBP0: Byte(0xE4),
            ..Default::default()
        };
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
        let mut ppu = PixelProcessor {
            LCDC: Byte(0x93),
            OBP0: Byte(0xE4), // id 3 -> shade 3
            OBP1: Byte(0x24), // id 3 -> shade 0
            ..Default::default()
        };
        solid_tile(&mut ppu, 1);
        // Two overlapping sprites; the lower-X one (slot 1) must win.
        place_sprite(&mut ppu, 0, 4, 0, 1, 0x10); // higher X, OBP1
        place_sprite(&mut ppu, 1, 2, 0, 1, 0x00); // lower X, OBP0

        ppu.render_sprite_line(0);

        // Overlap column 4: slot 1 (OBP0 -> shade 3) is on top.
        assert_eq!(ppu.frame[4], SHADES[3]);
    }
}
