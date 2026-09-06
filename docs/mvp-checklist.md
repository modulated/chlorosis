# MVP checklist — get a ROM rendering on screen

Goal: a background-only picture from a real ROM. Audio, save states, and
sprites are explicitly out of scope for this milestone.

Nothing renders today: `PixelProcessor::step_draw` is `unimplemented!()` and the
PPU never leaves HBlank. But several cheaper blockers panic the emulation thread
long before the renderer is even reached. Suggested order is bottom of this file.

## 1. Get the ROM into memory

- [ ] **Load the whole file.** Only `0x0100..=0x3FFF` is copied today, so the
  RST/interrupt vectors at `0x0000-0x00FF` are zero and banks 1+ are never
  filled. — `device.rs` `load_cartrige`
- [ ] **Size the ROM `Vec` from the header.** Hardcoded to 32 KB;
  `CartrigeHeader::rom_banks` is parsed and unused. — `device.rs::new`,
  `types/cartrige.rs`
- [ ] **Address arithmetic truncates to 16 bits.** `Address` wraps a `u16` and
  so does `Vec<Byte>` indexing, so `rom[addr + ROM_1_START * (bank-1)]` cannot
  reach past 64 KB. Needs a `usize` ROM-offset path. — `types/address.rs`,
  `device.rs::read/write`
- [ ] **Wire `mod mbc` into `read`/`write`.** ROM-region writes currently land
  in the ROM array instead of hitting bank registers; `set_cartrige_bank` has no
  callers. (Deferrable if testing with a 32 KB no-MBC ROM first.)

## 2. Stop the memory map killing the core thread

- [x] **Audio registers `0xFF10-0xFF3F` (+ `0xFF76/0xFF77`) `unimplemented!()`
  on read and write.** ROMs touch these within the first few hundred
  instructions. Stubbed: writes stored, reads returned. — `device.rs`
- [ ] **`0xFF50` (boot-ROM disable), `0xFF4C`, `0xFF4E`** route into
  `ppu.read_io/write_io`, which `unreachable!()`s on anything unlisted.
- [ ] **Echo RAM `0xE000-0xFDFF` and `0xFEA0-0xFEFF` `panic!`** — should mirror
  WRAM / return `0xFF`.

## 3. CPU

- [ ] **`step_cpu` runs 4× too fast.** Called every master tick, but `cost` is
  in M-cycles (`NOP => cost = 1`). Gate to every 4th tick, or convert costs to
  T-cycles. — `cpu/mod.rs`
- [ ] **Implement `HALT`, `RETI`, `DAA`** — all `unimplemented!()`. HALT is in
  essentially every main loop; RETI ends every interrupt handler. —
  `cpu/execute.rs`
- [ ] **Opcode correctness pass.** e.g. `0x02 LD (BC),A` reads instead of
  writing. Wants a Blargg `cpu_instrs` harness, not eyeballing. —
  `cpu/execute.rs`

## 4. Interrupts

- [ ] **IF (`0xFF0F`) and IE (`0xFFFF`) are the same field** — must be split.
- [ ] **No dispatch exists.** Nothing checks IME/IF/IE, pushes PC, or vectors to
  `0x40-0x60`. `EI`/`DI` set the flag and nothing reads it.
- [ ] **Raise interrupts** — VBlank + STAT from the PPU, timer overflow from the
  timer.

## 5. PPU — the actual rendering

- [ ] **`step_draw` is `unimplemented!()`** — no pixels produced, `buffer` never
  written. The single biggest item.
- [ ] **No mode state machine.** Never transitions HBlank→OAM→Draw; sits in
  HBlank counting LY forever. Needs the 80 / 172-289 / 87-204 dot schedule and
  LY 0-153.
- [ ] **LCDC bit-7 write is inverted.** `0xFF40` masks bit 7 off the value then
  the `else` calls `lcd_enable()`. — `ppu/registers.rs`
- [ ] **Tile-map area ranges disagree** — background returns VRAM-relative
  `0x1C00..`, window returns absolute `0x9C00..`. — `ppu/registers.rs`
- [x] **VRAM/OAM guards inverted and fatal.** `read_vram` panicked when the LCD
  was *enabled*; writes panicked outside HBlank/VBlank. Now keyed on the correct
  inaccessible modes (Draw for VRAM; OAM+Draw for OAM) and non-fatal: blocked
  reads return `0xFF`, blocked writes are ignored. — `ppu/mod.rs`
- [ ] **Pick a pixel path.** DMG (`BGP` + 4 greys) is far less work than CGB
  (`bcram`/`ocram`, BGR555). Convert to minifb's `0x00RRGGBB`.

## 6. Boot state

- [x] **Hardcode post-boot register state.** Only PC/SP were set; A/F/B/C/D/E/H/L
  were zero, `LCDC` was `0x00` (LCD off), `BGP` was `0x00`. Now seeded with CGB
  post-boot values (CPU registers, LCDC=0x91, BGP=0xFC, etc.) so the LCD starts
  on. Running the real boot ROM (`cgb_boot.bin`/`dmg_boot.bin` are committed but
  unused) and DMG-vs-CGB selection remain follow-ups. — `cpu/mod.rs`,
  `ppu/mod.rs`

## 7. Input

- [ ] **Joypad polarity inverted.** Hardware is active-low; unselected reads
  return `0x0F`. Currently pressed = 1 / select = bit set, so games read every
  button as held. — `joypad.rs`

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

**5 (audio stub) → 18 (VRAM/OAM guards) → 20 (boot state)** ← done
→ 11/12/13 (interrupts) → 8 (CPU timing) → 1/2/3 (ROM loading + addressing)
→ 15 (PPU modes) → 14/19 (renderer + palette) → 21 (joypad).
