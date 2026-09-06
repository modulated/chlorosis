# MVP checklist — get a ROM rendering on screen

Goal: a background-only picture from a real ROM. Audio, save states, and
sprites are explicitly out of scope for this milestone.

Nothing renders today: `PixelProcessor::step_draw` is `unimplemented!()` and the
PPU never leaves HBlank. But several cheaper blockers panic the emulation thread
long before the renderer is even reached. Suggested order is bottom of this file.

## 1. Get the ROM into memory

- [x] **Load the whole file.** `load_cartrige` now reads the entire file into
  the ROM image, so the RST/interrupt vectors at `0x0000-0x00FF` and every bank
  are present (a header smaller than 0x150 is rejected). — `device.rs`
- [x] **Size the ROM `Vec` from the header.** The image is resized to the
  header's declared size (never below the file, rounded up to a whole bank),
  padded with open-bus `0xFF`. Added `CartrigeHeader::rom_size()`. —
  `device.rs::load_cartrige`, `types/cartrige.rs`
- [x] **Address arithmetic truncated to 16 bits.** The switchable-bank read now
  computes `bank * 0x4000 + (addr - 0x4000)` in `usize` via `read_rom`, so it
  reaches past 64 KB; the old `Address` (u16) math wrapped. Out-of-range offsets
  return open-bus `0xFF`. — `device.rs::read`
- [x] **MBC wired into `read`/`write`.** A new `mbc::Mbc` (ROM-only, MBC1, MBC3,
  MBC5) maps the switchable ROM/RAM windows onto the flat image and absorbs the
  bank-select writes; `Device` builds it from the header's cartridge-type byte
  and sizes external RAM from the header. Banked ROMs larger than 32 KB now run
  in full. The old broken, unwired `mbc1/2/3/5.rs` are replaced. **MBC2 and
  MBC3's RTC are not modelled** (MBC2 falls back to MBC1 behaviour). — `mbc/mod.rs`,
  `device.rs`

## 2. Stop the memory map killing the core thread

- [x] **Audio registers `0xFF10-0xFF3F` (+ `0xFF76/0xFF77`) `unimplemented!()`
  on read and write.** ROMs touch these within the first few hundred
  instructions. Stubbed: writes stored, reads returned. — `device.rs`
- [ ] **`0xFF50` (boot-ROM disable), `0xFF4C`, `0xFF4E`** route into
  `ppu.read_io/write_io`, which `unreachable!()`s on anything unlisted.
- [ ] **Echo RAM `0xE000-0xFDFF` and `0xFEA0-0xFEFF` `panic!`** — should mirror
  WRAM / return `0xFF`.

## 3. CPU

- [x] **`step_cpu` ran 4× too fast.** It decremented `cost` (machine cycles)
  every master tick. `Device::tick` now steps the CPU once per four ticks, and
  a fetch/execute counts as the first machine cycle of the instruction's cost so
  totals match the tables. — `cpu/mod.rs`, `device.rs`
- [x] **`HALT` and `RETI`** implemented (they were `unimplemented!()`; HALT is in
  every main loop, RETI ends every handler). HALT idles until an enabled
  interrupt is pending; RETI pops PC and re-enables IME. **`DAA` and the
  IME-disabled HALT bug are still open** (part of the original item 9). —
  `cpu/execute.rs`
- [ ] **Opcode correctness pass.** e.g. `0x02 LD (BC),A` reads instead of
  writing. Wants a Blargg `cpu_instrs` harness, not eyeballing. —
  `cpu/execute.rs`

## 4. Interrupts

- [x] **IF (`0xFF0F`) and IE (`0xFFFF`) split** into separate fields — they were
  one, so each write clobbered the other. — `device.rs`
- [x] **Dispatch implemented.** At each instruction boundary the highest-priority
  pending+enabled interrupt (when IME is set) clears its IF bit, pushes PC, and
  vectors to `0x40-0x60`; it also wakes HALT. — `device.rs::service_interrupt`
- [x] **Interrupts raised.** Timer overflow reloads TMA and raises Timer; the PPU
  raises VBlank on entering line 144 and STAT for the enabled LYC/mode sources.
  — `timer.rs`, `ppu/mod.rs`

## 5. PPU — the actual rendering

- [x] **Background renderer.** On entering HBlank the scanline is drawn into a
  working frame; on entering VBlank the frame is published to the frontend.
  Background only for now — no window, no sprites, VRAM bank 0 — using SCX/SCY
  scroll, both tile-addressing modes, and honouring BG-enable (LCDC bit 0).
  **Sprites, the window, and the cycle-accurate pixel FIFO are the follow-ons.**
  — `ppu/mod.rs::render_background_line`
- [x] **Mode state machine (timing).** LY runs 0-153 and the mode cycles
  OAM(80)→Draw(172)→HBlank per visible line, VBlank at 144, driven per dot. This
  is what makes VBlank/STAT interrupts fire and drives the renderer. The Draw
  duration is fixed at the 172-dot minimum. — `ppu/mod.rs::step`
- [x] **LCDC bit-7 write inversion fixed.** `0xFF40` now stores the value
  directly; it used to mask bit 7 off and reconstruct it from an inverted
  condition, so clearing bit 7 outside VBlank turned the LCD on. —
  `ppu/registers.rs`
- [ ] **Tile-map area ranges disagree** — background returns VRAM-relative
  `0x1C00..` (which the renderer uses), window returns absolute `0x9C00..`. Fix
  when the window is rendered. — `ppu/registers.rs`
- [x] **VRAM/OAM guards inverted and fatal.** `read_vram` panicked when the LCD
  was *enabled*; writes panicked outside HBlank/VBlank. Now keyed on the correct
  inaccessible modes (Draw for VRAM; OAM+Draw for OAM) and non-fatal: blocked
  reads return `0xFF`, blocked writes are ignored. — `ppu/mod.rs`
- [x] **Pixel path (DMG).** The renderer maps BGP through a 4-shade greyscale
  ramp to minifb's `0x00RRGGBB`. CGB `bcram`/`ocram` BGR555 colour is a
  follow-on. — `ppu/mod.rs`

## 6. Boot state

- [x] **Hardcode post-boot register state.** Only PC/SP were set; A/F/B/C/D/E/H/L
  were zero, `LCDC` was `0x00` (LCD off), `BGP` was `0x00`. Now seeded with CGB
  post-boot values (CPU registers, LCDC=0x91, BGP=0xFC, etc.) so the LCD starts
  on. Running the real boot ROM (`cgb_boot.bin`/`dmg_boot.bin` are committed but
  unused) and DMG-vs-CGB selection remain follow-ups. — `cpu/mod.rs`,
  `ppu/mod.rs`

## 7. Input

- [x] **Joypad polarity fixed.** Reads are now active-low - a bit is 0 when
  pressed or its group selected, 1 otherwise, with unused bits 6-7 high and the
  idle lower nibble `0xF`. It used to report pressed = 1 and treat a set select
  bit as selected, so games saw every button held. **Joypad interrupt on
  key-down is still not raised** (fine for the many games that poll). —
  `joypad.rs`

## 8. Verification

- [ ] Headless harness: run N frames against a test ROM, hash the framebuffer.
- [ ] Blargg `cpu_instrs` and `dmg-acid2`.
- [ ] The two unit tests already failing on `main`
  (`cpu::arith::test::test_half_carry_sub_byte`, `types::cartrige::tests::read_header`)
  hint at real arithmetic/header bugs.

---

### Suggested order

Cheap unblockers first, so progress is observable rather than panicking on the
first audio write or a dark LCD:

**5 (audio stub) → 18 (VRAM/OAM guards) → 20 (boot state) → 11/12/13
(interrupts) → 8 (CPU timing) → 15 (PPU mode timing) → 1/2/3 (ROM loading +
addressing) → 14/19 (DMG background renderer) + 16 (LCDC write) → 21 (joypad) +
4 (MBC banking)** ← done

A ROM can now boot, display its background, and take input, with MBC1/3/5 games
running past 32 KB. The MVP path is complete; what remains is breadth and
accuracy:

- **Renderer follow-ons** — sprites (needs OAM DMA at `0xFF46`), the window
  (and the item-17 tile-map range fix), and CGB `bcram` colour.
- **9 (DAA, HALT bug)** and the **opcode correctness pass** (item 10), best
  driven by a Blargg `cpu_instrs` harness (item 22).
- **Accuracy leftovers** — the joypad interrupt, MBC2 / MBC3-RTC, and the
  remaining `panic!`ing memory holes (echo RAM, `0xFF50`, item-2 group).
