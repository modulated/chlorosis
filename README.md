# fungbc
GBC emulator

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
- 2 PWM oscs, 1 noise, 1 triangle