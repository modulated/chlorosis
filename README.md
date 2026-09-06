# Chlorosis
Gameboy and Gameboy Color emulator

## Architecture

The emulator and the window run on separate threads and never block on each
other.

```
  frontend thread                            emulation thread
  (debugger/src/main.rs)                     (chlorosis_core::Device)

    window events  --- Event ------------->  drained every frame
    window title   <-- CoreMessage --------  state, faults, speed
    presents       <-- frame swap ---------  published per PPU frame
```

* **`Event`** carries intents one way only. The frontend asks; it never
  mutates the emulator and never decides the emulator's state for it.
* **`CoreMessage`** carries what actually happened back: state changes,
  errors, faults, and a speed report. The emulator is the single authority on
  its own state, so the two sides cannot drift out of sync.
* **Frames** cross through a two slot swap (`chlorosis_core::framebuffer`)
  rather than a queue. Publishing overwrites any frame the frontend has not
  collected, so latency stays bounded at one frame however far the two rates
  drift, and steady state does not allocate.

The emulation thread schedules a frame at a time: 70,224 master clock ticks
back to back, then one sleep to the next 16.742 ms deadline, with deadlines
accumulated from a fixed origin so sleep overshoot does not compound. Falling
more than a few frames behind resets the deadline instead of sprinting to catch
up. The frontend paces itself independently and always redraws its last frame,
so a slow emulator makes the picture stale rather than making the window lag.

## Memory
- 32 KB Work RAM
- Cartrige space
- 16 KB Video RAM
- IO map
- Interrupt handlers

### Memory Map
`
  0000-3FFF   16KB ROM Bank 00     (in cartridge, fixed at bank 00)
  4000-7FFF   16KB ROM Bank 01..NN (in cartridge, switchable bank number)
  8000-9FFF   8KB Video RAM (VRAM) (switchable bank 0-1 in CGB Mode)
  A000-BFFF   8KB External RAM     (in cartridge, switchable bank, if any)
  C000-CFFF   4KB Work RAM Bank 0 (WRAM)
  D000-DFFF   4KB Work RAM Bank 1 (WRAM)  (switchable bank 1-7 in CGB Mode)
  E000-FDFF   Same as C000-DDFF (ECHO)    (typically not used)
  FE00-FE9F   Sprite Attribute Table (OAM)
  FEA0-FEFF   Not Usable
  FF00-FF7F   I/O Ports
  FF80-FFFE   High RAM (HRAM)
  FFFF        Interrupt Enable Register
`

- 0000,0008,0010,0018,0020,0028,0030,0038   for RST commands
- 0040,0048,0050,0058,0060                  for Interrupts
- cartrige header 0100-014F

## CPU
- Sharp LR35902
- Z80 based (without IX or IY registers)
- 8.4 MHz for GBC, 4.19 MHz for GB
- Memory mapped peripherals

## Video
- 160 x 144 pixels (20 x 18 tiles)
- 40 sprites max, 10 per line
- sprite size: 8 x 8 or 8 x 16
- H-sync 9198 KHz (9420 KHz for GB)
- V-sync 59.73 Hz (61.17 for GB)

## Sound
- 4 channel stereo
- 2 PWM oscs, 1 noise, 1 programmable oscillator
