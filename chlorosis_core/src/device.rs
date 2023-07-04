use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, Sender, TryRecvError},
    time::{Duration, Instant},
};

use super::{Address, Byte};

use crate::{constants::*, Event, Infrared, Joypad, KeyCode, Timer};

use super::{types::CartrigeHeader, AudioProcessor, CentralProcessor, PixelProcessor};

#[derive(Debug)]
pub struct Device {
    pub cpu: CentralProcessor,
    ppu: PixelProcessor,
    _audio: Option<AudioProcessor>,
    cartrige: Option<CartrigeHeader>,
    joypad: Joypad,
    rom: Vec<Byte>,
    wram: Vec<Byte>,
    eram: Vec<Byte>,
    hram: Vec<Byte>,
    interrupt: Byte,
    rom_bank: usize,
    wram_bank: Byte,
    infrared: Infrared,
    timer: Timer,
    state: DeviceState,
    rom_path: Option<PathBuf>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum DeviceState {
    Stopped,
    Running,
    Paused,
}

impl Device {
    pub fn new() -> Self {
        Self {
            cpu: CentralProcessor::default(),
            ppu: PixelProcessor::default(),
            _audio: None,
            cartrige: None,
            joypad: Joypad::default(),
            infrared: Infrared::default(),
            timer: Timer::default(),
            rom: vec![Byte(0); ROM_BANK_SIZE * 2], // TODO: need better way of determing ROM vec size
            wram: vec![Byte(0); WRAM_SIZE],
            eram: vec![Byte(0); ERAM_SIZE],
            rom_bank: 1,
            wram_bank: Byte(1),
            hram: vec![Byte(0); HRAM_SIZE],
            interrupt: Byte(0),
            rom_path: None,
            state: DeviceState::Stopped,
        }
    }

    pub fn run(&mut self, buffer: Sender<Vec<u32>>, event: Receiver<Event>) {
        // Outer loop should run at 4.1 MHz (or 8.2 if double speed enabled)

        let mut start = Instant::now();
        let mut last_frame = Instant::now();

        loop {
            match self.state {
                DeviceState::Stopped => self.stopped(&event),
                DeviceState::Running => self.running(&mut start, &mut last_frame, &buffer, &event),
                DeviceState::Paused => self.paused(&event),
            }
        }
    }

    fn running(
        &mut self,
        start: &mut Instant,
        last_frame: &mut Instant,
        buffer: &Sender<Vec<u32>>,
        event: &Receiver<Event>,
    ) {
        const TARGET: Duration = Duration::from_nanos(240); // roughly 240 ns per tick

        // Step CPU one cycle
        self.step_cpu();

        // Step PPU one cycle
        self.ppu.step();

        // Render audio

        let end = Instant::now();
        if (end - *last_frame) >= Duration::from_millis(16) {
            // Send
            if let Some(b) = &self.ppu.buffer {
                buffer.send(b.to_vec()).unwrap();
                self.ppu.buffer = None;
            }

            // Get events
            match event.try_recv() {
                Ok(event) => self.handle_event(event),
                Err(err) => match err {
                    TryRecvError::Disconnected => panic!("{err}"),
                    TryRecvError::Empty => {}
                },
            }

            *last_frame = end;
        }

        // Sleep as needed
        let dif = end - *start;
        if dif < TARGET {
            std::thread::sleep(TARGET - dif);
        }
        *start = end;
    }

    fn stopped(&mut self, event: &Receiver<Event>) {
        match event.try_recv() {
            Ok(event) => self.handle_event(event),
            Err(err) => match err {
                TryRecvError::Disconnected => panic!("{err}"),
                TryRecvError::Empty => {}
            },
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    fn paused(&mut self, event: &Receiver<Event>) {
        match event.try_recv() {
            Ok(event) => self.handle_event(event),
            Err(err) => match err {
                TryRecvError::Disconnected => panic!("{err}"),
                TryRecvError::Empty => {}
            },
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    fn reset(&mut self) {
        let rom = self.rom_path.clone();
        *self = Self::new();
        if let Some(rom) = rom {
            self.load_cartrige(rom).unwrap();
        }
    }

    pub fn load_cartrige(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), std::io::Error> {
        use std::io::Read;
        let mut f = std::fs::File::open(path)?;
        let mut buf = vec![];
        f.read_to_end(&mut buf)?;

        let mut iter = buf.iter().skip(0x0100);
        println!("Reading cartrige, {} bytes", buf.len());
        for i in 0x0100..=ROM_0_END {
            self.rom[i as usize] = Byte(*iter.next().expect("Early end to cartrige"));
        }
        self.cartrige = Some(CartrigeHeader::from_bytes(self.get_header()));
        self.dump_cartrige_header();
        // TODO: read rest of ROM

        self.state = DeviceState::Running;

        Ok(())
    }

    pub fn read(&mut self, address: Address) -> Byte {
        match address.0 {
            ROM_0_START..=ROM_0_END => self.rom[address],
            ROM_1_START..=ROM_1_END => {
                self.rom[address + Address(ROM_1_START) * (self.rom_bank - 1)]
            }
            VRAM_START..=VRAM_END => self.ppu.read_vram(address),
            ERAM_START..=ERAM_END => self.eram[address - Address(ERAM_START)], // External ram
            WRAM_0_START..=WRAM_0_END => self.wram[address - Address(WRAM_0_START)],
            WRAM_1_START..=WRAM_1_END => {
                self.wram[address + (Address(WRAM_BANK_SIZE as u16) * self.wram_bank.0 as usize)
                    - Address(WRAM_1_START)]
            }
            DEADZONE_0_START..=DEADZONE_0_END => panic!("Prohibited memory access at {address}"),
            OAM_START..=OAM_END => self.ppu.read_oam(address),
            DEADZONE_1_START..=DEADZONE_1_END => panic!("Prohibited memory access at {address}"),

            // IO START
            0xFF00 => self.joypad.read(), // Joypad
            0xFF01..=0xFF02 => Byte(0),   // TODO: Serial
            0xFF03 => panic!("Prohibited memory access at {address}"), // Prohibited
            0xFF04..=0xFF07 => self.timer.read(address), // Timers
            0xFF08..=0xFF0E => panic!("Prohibited memory access at {address}"), // Prohibited
            0xFF0F => self.interrupt,     // Interrupt
            0xFF10..=0xFF3F => unimplemented!("Audio"), // Audio
            0xFF40..=0xFF55 => self.ppu.read_io(address), // PPU
            0xFF56 => self.infrared.read(), // Infrared Com Port
            0xFF57..=0xFF6F => self.ppu.read_io(address), // PPU
            0xFF70 => self.wram_bank,     // WRAM BANK
            0xFF71..=0xFF75 => panic!("Prohibited memory access at {address}"), // Prohibited
            0xFF76 => unimplemented!("Audio"), // Audio 1&2
            0xFF77 => unimplemented!("Audio"), // Audio 3&4
            0xFF78..=0xFF7F => panic!("Prohibited memory access at {address}"), // Prohibited
            // IO END
            HRAM_START..=HRAM_END => self.hram[address - Address(HRAM_START)],
            INTERRUPT_ENABLE => self.interrupt,
        }
    }

    pub fn write(&mut self, address: Address, value: Byte) {
        match address.0 {
            ROM_0_START..=ROM_0_END => self.rom[address] = value,
            ROM_1_START..=ROM_1_END => {
                self.rom[address + Address(ROM_1_START) * (self.rom_bank - 1)] = value
            }
            VRAM_START..=VRAM_END => {
                self.ppu.write_vram(address, value);
            }
            ERAM_START..=ERAM_END => self.eram[address - Address(ERAM_START)] = value, // External ram
            WRAM_0_START..=WRAM_0_END => self.wram[address - Address(WRAM_0_START)] = value,
            WRAM_1_START..=WRAM_1_END => {
                self.wram[address + (Address(WRAM_BANK_SIZE as u16) * self.wram_bank.0 as usize)
                    - Address(WRAM_1_START)] = value
            }
            DEADZONE_0_START..=DEADZONE_0_END => panic!("Prohibited memory access at {address}"),
            OAM_START..=OAM_END => self.ppu.write_oam(address, value),
            DEADZONE_1_START..=DEADZONE_1_END => panic!("Prohibited memory access at {address}"),

            // IO_START
            0xFF00 => self.joypad.write(value), // Joypad
            0xFF01..=0xFF02 => {}               // TODO: Serial
            0xFF03 => panic!("Prohibited memory access at {address}"), // Prohibited
            0xFF04..=0xFF07 => self.timer.write(address, value), // Timers
            0xFF08..=0xFF0E => panic!("Prohibited memory access at {address}"), // Prohibited
            0xFF0F => self.interrupt = value,   // Interrupt
            0xFF10..=0xFF3F => unimplemented!("Audio"), // Audio
            0xFF40..=0xFF55 => self.ppu.write_io(address, value), // PPU
            0xFF56 => self.infrared.write(value), // Infrared Com Port
            0xFF57..=0xFF6F => self.ppu.write_io(address, value), // PPU
            0xFF70 => self.wram_bank = value,   // WRAM BANK
            0xFF71..=0xFF75 => panic!("Prohibited memory access at {address}"), // Prohibited
            0xFF76 => unimplemented!("Audio"),  // Audio channels 1 & 2,
            0xFF77 => unimplemented!("Audio"),  // Audio channels 3 & 4,
            0xFF78..=0xFF7F => panic!("Prohibited memory access at {address}"), // Prohibited
            // IO END
            HRAM_START..=HRAM_END => self.hram[address - Address(HRAM_START)] = value,
            INTERRUPT_ENABLE => self.interrupt = value,
        }
    }

    pub fn set_cartrige_bank(&mut self, value: usize) {
        self.rom_bank = value;
    }

    pub fn get_header(&self) -> &[Byte] {
        &self.rom[0x100..=0x14F]
    }

    pub fn dump_rom(&mut self) {
        for i in ROM_0_START..=ROM_1_END {
            let byte = self.read(Address(i));
            if i % 32 == 0 {
                println!();
                print!("{}: ", Address(i));
            }
            if i % 8 == 0 {
                print!("  ");
            }
            print!("{} ", byte);
        }
    }

    pub fn dump_cpu(&self) {
        println!("CPU State: ");
        self.cpu.dump_state();
    }

    pub fn dump_cartrige_header(&self) {
        self.cartrige
            .as_ref()
            .map_or_else(|| println!("No cartrige loaded"), |c| println!("{c:#?}"))
    }

    fn handle_event(&mut self, event: Event) {
        println!("{event:?}");
        match event {
            Event::KeyDown(k) => self.handle_keydown(k),
            Event::KeyUp(k) => self.handle_keyup(k),
            Event::LoadFile(f) => self.load_cartrige(f).unwrap(),
            Event::Pause => self.state = DeviceState::Paused,
            Event::Run => self.state = DeviceState::Running,
            Event::Reset => self.reset(),
            Event::Exit => std::process::exit(0),
            _ => unimplemented!("Unimplemented event {event:?}"),
        }
    }

    fn handle_keydown(&mut self, keys: Vec<KeyCode>) {
        for b in keys {
            self.joypad.press(b);
        }
    }

    fn handle_keyup(&mut self, keys: Vec<KeyCode>) {
        for b in keys {
            self.joypad.release(b);
        }
    }
}

impl Default for Device {
    fn default() -> Self {
        Self::new()
    }
}
