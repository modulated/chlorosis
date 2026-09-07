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
- [x] **MBC wired into `read`/`write`.** A new `mbc::Mbc` (ROM-only, MBC1, MBC2,
  MBC3, MBC5) maps the switchable ROM/RAM windows onto the flat image and absorbs
  the bank-select writes; `Device` builds it from the header's cartridge-type
  byte and sizes external RAM from the header. Banked ROMs larger than 32 KB now
  run in full. MBC2's 4-bit ROM bank register (address bit 8) and built-in
  512×4-bit RAM are modelled, and MBC3's real-time clock ticks on emulated
  cycles with latch and halt. RAM/RTC access routes through
  `Mbc::read_ram`/`write_ram`. — `mbc/mod.rs`, `device.rs`

## 2. Stop the memory map killing the core thread

- [x] **Audio registers `0xFF10-0xFF3F` (+ `0xFF76/0xFF77`).** Now a full APU:
  two square channels (one with sweep), the wave channel, and the noise channel,
  with envelopes, length counters, and a 512 Hz frame sequencer, mixed to stereo
  and resampled for the host. Played through cpal behind the default `audio`
  feature. — `audio.rs`, `debugger/src/audio.rs`
- [x] **`0xFF50` (boot-ROM disable), `0xFF4C`, `0xFF4E`, and the prohibited IO
  ranges** used to `panic!`/`unreachable!()`. Unlisted registers in the PPU's
  routed ranges now read open bus and drop writes, and the prohibited holes
  (`0xFF03`, `0xFF08-0E`, `0xFF71-75`, `0xFF78-7F`) do the same in `device.rs`.
  A stray IO access can no longer fault the core. — `device.rs`, `ppu/registers.rs`
- [x] **Echo RAM `0xE000-0xFDFF` and `0xFEA0-0xFEFF`** — echo now mirrors WRAM
  `0x2000` below it (read and write); the unusable region reads `0xFF` and drops
  writes instead of panicking. — `device.rs`

## 3. CPU

- [x] **`step_cpu` ran 4× too fast.** It decremented `cost` (machine cycles)
  every master tick. `Device::tick` now steps the CPU once per four ticks, and
  a fetch/execute counts as the first machine cycle of the instruction's cost so
  totals match the tables. — `cpu/mod.rs`, `device.rs`
- [x] **`HALT` and `RETI`** implemented (they were `unimplemented!()`; HALT is in
  every main loop, RETI ends every handler). HALT idles until an enabled
  interrupt is pending; RETI pops PC and re-enables IME. `DAA` is now
  implemented too, and so is the IME-disabled HALT bug: a `HALT` with IME
  clear and an interrupt already pending no longer halts - the following opcode
  byte is fetched twice (PC fails to advance once). — `cpu/execute.rs`,
  `cpu/fetch.rs`
- [x] **Opcode correctness pass.** All eleven Blargg `cpu_instrs` tests pass,
  driven by a gameboy-doctor-style trace diff against a reference log. Fixed:
  `to_signed` (every relative jump), 8/16-bit arithmetic wraparound, `DEC DE`,
  ADD/ADC flags, the subtract-carry helper, `ADD A,d8`, `RRA`/`RLA` write-back,
  `DEC E` mis-decode, the CB rotate/shift flags and A-register variants,
  `ADD SP,e8` (wrote PC), `JP NZ` (inverted), `RET C` (popped SP), and a
  double cost decrement. — `cpu/`

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

- [x] **Background, sprite, and window renderers.** On entering HBlank the
  scanline is drawn into a working frame (background, then window, then
  sprites); on entering VBlank the frame is published. Uses SCX/SCY scroll,
  both tile-addressing modes, and honours BG-enable (LCDC bit 0). The window is
  drawn at its WX/WY position with its own line counter. **The cycle-accurate
  pixel FIFO is the remaining follow-on.** — `ppu/mod.rs`
- [x] **Mode state machine (timing).** LY runs 0-153 and the mode cycles
  OAM(80)→Draw(172)→HBlank per visible line, VBlank at 144, driven per dot. This
  is what makes VBlank/STAT interrupts fire and drives the renderer. The Draw
  duration is fixed at the 172-dot minimum. — `ppu/mod.rs::step`
- [x] **LCDC bit-7 write inversion fixed.** `0xFF40` now stores the value
  directly; it used to mask bit 7 off and reconstruct it from an inverted
  condition, so clearing bit 7 outside VBlank turned the LCD on. —
  `ppu/registers.rs`
- [x] **Tile-map area ranges disagree** — background returns VRAM-relative
  `0x1C00..`, window returns absolute `0x9C00..`. The window renderer subtracts
  `VRAM_START` so both index VRAM correctly. — `ppu/registers.rs`, `ppu/mod.rs`
- [x] **VRAM/OAM guards inverted and fatal.** `read_vram` panicked when the LCD
  was *enabled*; writes panicked outside HBlank/VBlank. Now keyed on the correct
  inaccessible modes (Draw for VRAM; OAM+Draw for OAM) and non-fatal: blocked
  reads return `0xFF`, blocked writes are ignored. — `ppu/mod.rs`
- [x] **Pixel path (DMG and CGB).** DMG maps BGP/OBP through a 4-shade greyscale
  ramp; CGB colour reads the per-tile attributes from VRAM bank 1 (palette,
  bank, flip, priority) and maps `bcram`/`ocram` BGR555 to `0x00RRGGBB`. The
  renderer is chosen from the cartridge CGB flag on load. — `ppu/mod.rs`

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
  bit as selected, so games saw every button held. The Joypad interrupt is now
  raised too: pressing a button on a selected line (or selecting a group that
  holds one) drives a P10-P13 line high to low and requests it. — `joypad.rs`,
  `device.rs`

## 8. Verification

- [x] Headless harness: `tests/serial_harness.rs` drives a ROM through
  `Device::tick` and reads the serial console; it runs a real Blargg ROM when
  `CHLOROSIS_TEST_ROM` points to one, and includes a self-contained CPU→serial
  test otherwise.
- [x] **Blargg `cpu_instrs`: all 11 individual tests pass.** `dmg-acid2` (PPU
  accuracy) is still untried.
- [x] `test_half_carry_sub_byte` was a wrong assertion, now corrected;
  `read_header` still fails only because its ROM file is absent from the repo.

---

### Suggested order

Cheap unblockers first, so progress is observable rather than panicking on the
first audio write or a dark LCD:

**5 (audio stub) → 18 (VRAM/OAM guards) → 20 (boot state) → 11/12/13
(interrupts) → 8 (CPU timing) → 15 (PPU mode timing) → 1/2/3 (ROM loading +
addressing) → 14/19 (DMG background renderer) + 16 (LCDC write) → 21 (joypad) +
4 (MBC banking)** ← done

A ROM can now boot, display its background, window, and sprites in DMG
greyscale or CGB colour, take input, run past 32 KB via MBC1/3/5, save state
and restore, keep battery-backed RAM across runs, and pass every Blargg
`cpu_instrs` test. The MVP path is complete and the CPU is validated; what
remains is breadth and accuracy:

- **Renderer follow-on** — the cycle-accurate pixel FIFO. Background, window,
  sprites, and CGB colour (verified against `cgb-acid2`) are done.
- **PPU/timing accuracy** — Blargg's timing tests (`instr_timing`,
  `mem_timing`), and CGB double-speed.
- **Accuracy leftovers** — audio *accuracy* (the channels play, but timing is
  not cycle-exact and the DMG/CGB power-off quirks are approximated), and
  cycle-exact PPU/instruction timing. The MBC3 RTC keeps its state in save
  states but is not yet appended to the `.sav` file.

Since this list was written, the window layer, the CGB colour renderer, the
tile-map range fix, echo RAM, the IO-hole faults, the Joypad interrupt, the
IME-disabled HALT bug, MBC2 and the MBC3 RTC, a full **APU with cpal output**,
**save states** (with slot/quick keys and a validated file format), and
**battery-backed `.sav` persistence** have all landed — the last two were
originally out of scope for the MVP.
