use std::{
    any::Any,
    panic::{catch_unwind, AssertUnwindSafe},
    path::PathBuf,
    sync::mpsc::TryRecvError,
    time::{Duration, Instant},
};

use super::{Address, Byte};

use crate::{
    constants::*, mbc::Mbc, CoreChannels, CoreMessage, Event, Infrared, Joypad, KeyCode, Timer,
};

use super::{types::CartrigeHeader, AudioProcessor, CentralProcessor, PixelProcessor};

/// Master clock ticks in one video frame: 154 lines of 456 dots.
pub const TICKS_PER_FRAME: u32 = 70_224;

/// 4.194304 MHz / 70224 ticks == 59.7275 Hz.
const FRAME_TIME: Duration = Duration::from_nanos(16_742_706);

/// First audio register, `NR10`.
const AUDIO_REG_START: u16 = 0xFF10;
/// Number of audio registers, `0xFF10..=0xFF3F` (control regs plus wave RAM).
const AUDIO_REG_COUNT: usize = 0x30;

/// External-RAM bank size (8 KB), used to size `eram` and count RAM banks.
const RAM_BANK_SIZE: usize = 0x2000;

/// How far behind real time the emulator may fall before it stops trying to
/// catch up. Without this a long host stall (a dragged window, a swapped out
/// page) leaves a debt the emulator repays by sprinting through several frames
/// of gameplay at once.
const MAX_CATCHUP_FRAMES: u32 = 4;

/// How often throughput is reported to the frontend.
const SPEED_REPORT_INTERVAL: Duration = Duration::from_secs(1);

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
    /// Interrupt Flag (`0xFF0F`): which interrupts are currently requested.
    interrupt_flag: Byte,
    /// Interrupt Enable (`0xFFFF`): which requested interrupts may be serviced.
    /// These were previously one shared field, so IF and IE aliased each other.
    interrupt_enable: Byte,
    /// Which master tick of the current machine cycle we are on (0..4). The CPU
    /// advances once every four ticks; the PPU and timer advance every tick.
    mcycle_phase: u8,
    /// Cartridge memory-bank controller: maps the ROM/RAM windows onto `rom`
    /// and `eram`, and absorbs the bank-select writes into the ROM range.
    mbc: Mbc,
    wram_bank: Byte,
    infrared: Infrared,
    timer: Timer,
    /// Raw store for the audio registers `0xFF10-0xFF3F`. Audio is out of scope
    /// for now, but ROMs write these within the first few hundred instructions
    /// and read some of them back, so the values are kept rather than acted on.
    /// Once a real `AudioProcessor` exists this store moves into it.
    audio_regs: [Byte; AUDIO_REG_COUNT],
    state: EmulatorState,
    rom_path: Option<PathBuf>,
}

/// Whether the emulator is executing. The emulation thread is the only owner of
/// this; the frontend learns about changes through [`CoreMessage::State`]
/// rather than tracking its own copy.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EmulatorState {
    /// No cartridge loaded.
    Stopped,
    Running,
    Paused,
}

/// Whether the emulation loop should keep going after handling an event.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Control {
    Continue,
    Shutdown,
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
            audio_regs: [Byte(0); AUDIO_REG_COUNT],
            rom: vec![Byte(0); ROM_BANK_SIZE * 2], // resized to the cartridge on load
            wram: vec![Byte(0); WRAM_SIZE],
            eram: vec![Byte(0); ERAM_SIZE],
            // ROM-only default; replaced from the header when a cartridge loads.
            mbc: Mbc::new(0x00, 2, ERAM_SIZE / RAM_BANK_SIZE),
            wram_bank: Byte(1),
            hram: vec![Byte(0); HRAM_SIZE],
            interrupt_flag: Byte(0),
            interrupt_enable: Byte(0),
            mcycle_phase: 0,
            rom_path: None,
            state: EmulatorState::Stopped,
        }
    }

    /// Run the emulator until the frontend asks it to stop. Intended as the
    /// body of a dedicated thread.
    ///
    /// The memory map still reaches `panic!` and `unimplemented!` in plenty of
    /// places, and a ROM only has to touch one of them to kill this thread. If
    /// that were allowed to happen silently the frontend would keep presenting
    /// a frozen picture with no explanation, so the panic is caught and
    /// reported before the thread winds down.
    pub fn run(&mut self, channels: CoreChannels) {
        let messages = channels.messages.clone();
        if let Err(panic) = catch_unwind(AssertUnwindSafe(|| self.run_loop(&channels))) {
            // The frontend may already be gone, in which case there is nobody
            // left to tell and nothing to do about it.
            let _ = messages.send(CoreMessage::Faulted(describe_panic(panic.as_ref())));
        }
    }

    fn run_loop(&mut self, channels: &CoreChannels) {
        let mut pacer = Pacer::new();

        loop {
            let control = match self.state {
                // Nothing to emulate, so block rather than poll: the thread
                // costs nothing while idle and still reacts to the next event
                // the instant it arrives, instead of up to a sleep later.
                EmulatorState::Stopped | EmulatorState::Paused => {
                    let control = self.wait_for_event(channels);
                    pacer.resume();
                    control
                }
                EmulatorState::Running => self.run_one_frame(channels, &mut pacer),
            };

            if control == Control::Shutdown {
                return;
            }
        }
    }

    /// Emulate exactly one frame, then sleep until that frame's worth of real
    /// time has elapsed.
    ///
    /// The frame is the scheduling quantum on purpose. Pacing per tick means
    /// asking the OS to sleep for 240 ns 70,224 times a frame, and no
    /// general purpose scheduler will wake you that precisely - each of those
    /// sleeps overshoots by tens of microseconds, so the emulator ends up
    /// running orders of magnitude below real speed. One sleep per frame is
    /// both accurate enough to hit 59.7275 Hz and cheap enough to be free.
    fn run_one_frame(&mut self, channels: &CoreChannels, pacer: &mut Pacer) -> Control {
        // Drain the whole queue, not one event per frame: input arrives in
        // press/release pairs, and taking a single event per frame turns a
        // tap into a backlog that grows for as long as the player keeps
        // playing.
        if self.drain_events(channels) == Control::Shutdown {
            return Control::Shutdown;
        }

        // An event may have halted us; let the loop re-dispatch rather than
        // emulating one more frame the user did not ask for.
        if self.state != EmulatorState::Running {
            return Control::Continue;
        }

        self.tick(TICKS_PER_FRAME);
        self.publish_frame(channels);

        if let Some(report) = pacer.frame_completed() {
            if channels.messages.send(report).is_err() {
                return Control::Shutdown;
            }
        }

        Control::Continue
    }

    /// Advance every component by `ticks` of the 4.19 MHz master clock.
    fn tick(&mut self, ticks: u32) {
        for _ in 0..ticks {
            // The PPU and timer run on the master clock, one step per tick.
            let mut pending = self.ppu.step();
            if self.timer.tick() {
                pending |= Interrupts::Timer;
            }
            self.request_interrupts(pending);

            // The CPU runs on the machine clock: one step every fourth tick.
            self.mcycle_phase += 1;
            if self.mcycle_phase == 4 {
                self.mcycle_phase = 0;
                self.step_cpu();
            }
        }
    }

    /// Flag `ints` as requested, for the CPU to service when interrupts are
    /// enabled.
    const fn request_interrupts(&mut self, ints: Interrupts) {
        self.interrupt_flag.0 |= ints.bits();
    }

    /// Service the highest-priority pending, enabled interrupt if IME is set,
    /// and wake the CPU from HALT for any pending, enabled interrupt regardless
    /// of IME. Returns `true` if an interrupt was dispatched.
    pub(crate) fn service_interrupt(&mut self) -> bool {
        let pending = self.interrupt_flag.0 & self.interrupt_enable.0 & 0x1F;
        if pending == 0 {
            return false;
        }

        // A pending, enabled interrupt ends HALT even when IME is clear; the CPU
        // simply resumes without vectoring.
        self.cpu.halted = false;

        if !self.cpu.interupt_master_enable {
            return false;
        }

        // The lowest set bit is the highest priority (VBlank first).
        let index = pending.trailing_zeros() as usize;
        self.interrupt_flag.0 &= !(1 << index);
        self.cpu.interupt_master_enable = false;
        self.push_address(self.cpu.pc);
        self.cpu.pc = Address(INTERRUPT_VECTORS[index]);
        // Dispatch takes five machine cycles; this call is the first.
        self.cpu.cost = 4;
        true
    }

    /// Hand the frontend a finished frame, if the PPU produced one.
    ///
    /// Frames go out when the PPU says a frame is done rather than on a wall
    /// clock, so what the frontend shows is whole frames and not whatever the
    /// PPU happened to have drawn when a timer expired.
    fn publish_frame(&mut self, channels: &CoreChannels) {
        if let Some(rendered) = self.ppu.take_frame() {
            let mut frame = channels.frames.acquire();
            frame.copy_from_slice(&rendered);
            channels.frames.publish(frame);
        }
    }

    fn drain_events(&mut self, channels: &CoreChannels) -> Control {
        loop {
            match channels.events.try_recv() {
                Ok(event) => {
                    if self.handle_event(event, channels) == Control::Shutdown {
                        return Control::Shutdown;
                    }
                }
                Err(TryRecvError::Empty) => return Control::Continue,
                // The frontend is gone; there is nothing left to emulate for.
                Err(TryRecvError::Disconnected) => return Control::Shutdown,
            }
        }
    }

    fn wait_for_event(&mut self, channels: &CoreChannels) -> Control {
        channels.events.recv().map_or(Control::Shutdown, |event| {
            self.handle_event(event, channels)
        })
    }

    fn handle_event(&mut self, event: Event, channels: &CoreChannels) -> Control {
        match event {
            Event::KeyDown(keys) => self.handle_keydown(keys),
            Event::KeyUp(keys) => self.handle_keyup(keys),
            Event::LoadFile(path) => self.handle_load_file(&path, channels),
            Event::Run => {
                if self.cartrige.is_some() {
                    self.set_state(EmulatorState::Running, channels);
                } else {
                    report(channels, CoreMessage::Error("No cartrige loaded".to_owned()));
                }
            }
            Event::Pause => {
                if self.state == EmulatorState::Running {
                    self.set_state(EmulatorState::Paused, channels);
                }
            }
            Event::Step(ticks) => {
                // Stepping is only meaningful while halted; while running the
                // emulator is already advancing on its own.
                if self.state == EmulatorState::Paused {
                    self.tick(ticks);
                    self.publish_frame(channels);
                }
            }
            Event::Reset => self.handle_reset(channels),
            // Unsupported requests are reported, never panicked on: a menu item
            // the frontend has not finished wiring up must not be able to take
            // the emulation thread down with it.
            Event::SaveState(_) | Event::LoadState(_) => report(
                channels,
                CoreMessage::Error("Save states are not implemented".to_owned()),
            ),
            Event::Exit => return Control::Shutdown,
        }
        Control::Continue
    }

    fn handle_load_file(&mut self, path: &std::path::Path, channels: &CoreChannels) {
        match self.load_cartrige(path) {
            Ok(()) => {
                let title = self
                    .cartrige
                    .as_ref()
                    .map_or("Unknown", CartrigeHeader::title)
                    .to_owned();
                report(channels, CoreMessage::CartridgeLoaded(title));
                self.set_state(EmulatorState::Running, channels);
            }
            Err(e) => report(
                channels,
                CoreMessage::Error(format!("Could not load {}: {e}", path.display())),
            ),
        }
    }

    fn handle_reset(&mut self, channels: &CoreChannels) {
        match self.reset() {
            Ok(()) => {
                let state = if self.cartrige.is_some() {
                    EmulatorState::Running
                } else {
                    EmulatorState::Stopped
                };
                self.set_state(state, channels);
            }
            Err(e) => {
                report(channels, CoreMessage::Error(format!("Reset failed: {e}")));
                self.set_state(EmulatorState::Stopped, channels);
            }
        }
    }

    fn set_state(&mut self, state: EmulatorState, channels: &CoreChannels) {
        if self.state != state {
            self.state = state;
            report(channels, CoreMessage::State(state));
        }
    }

    pub const fn state(&self) -> EmulatorState {
        self.state
    }

    /// Restore power on state, keeping whatever cartridge is inserted.
    fn reset(&mut self) -> Result<(), std::io::Error> {
        let rom = self.rom_path.clone();
        *self = Self::new();
        rom.map_or(Ok(()), |rom| self.load_cartrige(rom))
    }

    pub fn load_cartrige(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), std::io::Error> {
        use std::io::Read;
        let path = path.as_ref();
        let mut f = std::fs::File::open(path)?;
        let mut buf = vec![];
        f.read_to_end(&mut buf)?;
        println!("Reading cartrige, {} bytes", buf.len());

        // Load the entire file, not just 0x0100..=0x3FFF. The old copy left the
        // interrupt/RST vectors at 0x0000-0x00FF and every bank past the first
        // as zeros - so a jump to a vector, or any code above bank 0, ran into
        // blank memory.
        if buf.len() < 0x0150 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "cartrige is too small to contain a header",
            ));
        }
        self.rom = buf.into_iter().map(Byte).collect();

        // Size the ROM image from the header, so bank arithmetic addresses a
        // buffer that is actually large enough. A ROM should already be its
        // declared size and a whole number of banks; pad with open-bus 0xFF if
        // it falls short rather than panicking on a read.
        let header = CartrigeHeader::from_bytes(self.get_header());
        let declared = header.rom_size() as usize;
        let sized = declared
            .max(self.rom.len())
            .next_multiple_of(ROM_BANK_SIZE);
        self.rom.resize(sized, Byte(0xFF));

        // Size external RAM from the header and build the controller named by
        // the cartridge-type byte, so the switchable ROM/RAM windows map onto
        // the real image.
        let ram_size = (header.ram_size() as usize).next_multiple_of(RAM_BANK_SIZE);
        self.eram = vec![Byte(0); ram_size];
        let cartridge_type = self.rom[0x0147].0;
        self.mbc = Mbc::new(
            cartridge_type,
            self.rom.len() / ROM_BANK_SIZE,
            self.eram.len() / RAM_BANK_SIZE,
        );

        self.cartrige = Some(header);
        self.rom_path = Some(path.to_path_buf());
        self.dump_cartrige_header();

        Ok(())
    }

    pub const fn get_cartridge_header(&self) -> Option<&CartrigeHeader> {
        self.cartrige.as_ref()
    }

    /// Read a byte from the flat ROM image at a physical `offset`, returning
    /// open-bus `0xFF` for offsets past the end (an out-of-range bank, or a ROM
    /// smaller than its header claims).
    fn read_rom(&self, offset: usize) -> Byte {
        self.rom.get(offset).copied().unwrap_or(Byte(0xFF))
    }

    pub fn read(&mut self, address: Address) -> Byte {
        match address.0 {
            // Both ROM windows go through the MBC, which maps them onto the flat
            // ROM image in `usize` (so banks past 64 KB no longer wrap) and
            // selects the switchable bank.
            ROM_0_START..=ROM_1_END => self.read_rom(self.mbc.rom_offset(address.0)),
            VRAM_START..=VRAM_END => self.ppu.read_vram(address),
            // External cartridge RAM, if the MBC has it mapped and enabled;
            // otherwise the bus floats to 0xFF.
            ERAM_START..=ERAM_END => self
                .mbc
                .ram_offset(address.0)
                .and_then(|o| self.eram.get(o).copied())
                .unwrap_or(Byte(0xFF)),
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
            0xFF0F => Byte(self.interrupt_flag.0 | 0xE0), // IF (top 3 bits read as 1)
            0xFF10..=0xFF3F => self.audio_regs[(address.0 - AUDIO_REG_START) as usize], // Audio
            0xFF40..=0xFF55 => self.ppu.read_io(address), // PPU
            0xFF56 => self.infrared.read(), // Infrared Com Port
            0xFF57..=0xFF6F => self.ppu.read_io(address), // PPU
            0xFF70 => self.wram_bank,     // WRAM BANK
            0xFF71..=0xFF75 => panic!("Prohibited memory access at {address}"), // Prohibited
            0xFF76 => Byte(0),            // Audio PCM12 (read-only, no audio yet)
            0xFF77 => Byte(0),            // Audio PCM34 (read-only, no audio yet)
            0xFF78..=0xFF7F => panic!("Prohibited memory access at {address}"), // Prohibited
            // IO END
            HRAM_START..=HRAM_END => self.hram[address - Address(HRAM_START)],
            INTERRUPT_ENABLE => self.interrupt_enable,
        }
    }

    pub fn write(&mut self, address: Address, value: Byte) {
        match address.0 {
            // Writes into the ROM range never store into ROM; they program the
            // MBC's bank-select and control registers.
            ROM_0_START..=ROM_1_END => self.mbc.write_control(address.0, value.0),
            VRAM_START..=VRAM_END => {
                self.ppu.write_vram(address, value);
            }
            // External cartridge RAM, if mapped and enabled; dropped otherwise.
            ERAM_START..=ERAM_END => {
                if let Some(offset) = self.mbc.ram_offset(address.0) {
                    if let Some(cell) = self.eram.get_mut(offset) {
                        *cell = value;
                    }
                }
            }
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
            0xFF0F => self.interrupt_flag = Byte(value.0 & 0x1F), // IF
            0xFF10..=0xFF3F => self.audio_regs[(address.0 - AUDIO_REG_START) as usize] = value, // Audio
            0xFF40..=0xFF55 => self.ppu.write_io(address, value), // PPU
            0xFF56 => self.infrared.write(value), // Infrared Com Port
            0xFF57..=0xFF6F => self.ppu.write_io(address, value), // PPU
            0xFF70 => self.wram_bank = value,   // WRAM BANK
            0xFF71..=0xFF75 => panic!("Prohibited memory access at {address}"), // Prohibited
            0xFF76 => {}                        // Audio PCM12 (read-only)
            0xFF77 => {}                        // Audio PCM34 (read-only)
            0xFF78..=0xFF7F => panic!("Prohibited memory access at {address}"), // Prohibited
            // IO END
            HRAM_START..=HRAM_END => self.hram[address - Address(HRAM_START)] = value,
            INTERRUPT_ENABLE => self.interrupt_enable = value,
        }
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

/// Sends a report to the frontend, tolerating its absence. A frontend that has
/// already shut down is not an emulation error, and the loop notices the
/// disconnect on its next event drain anyway.
fn report(channels: &CoreChannels, message: CoreMessage) {
    let _ = channels.messages.send(message);
}

fn describe_panic(payload: &(dyn Any + Send)) -> String {
    payload.downcast_ref::<&'static str>().map_or_else(
        || {
            payload
                .downcast_ref::<String>()
                .cloned()
                .unwrap_or_else(|| "emulation thread panicked".to_owned())
        },
        |s| (*s).to_owned(),
    )
}

/// Keeps emulated time aligned with real time.
///
/// Deadlines accumulate from a fixed origin rather than being recomputed from
/// "now" each frame, so the microseconds a sleep overshoots by are absorbed by
/// the next frame instead of compounding into visible drift.
#[derive(Debug)]
struct Pacer {
    next_frame: Instant,
    window_started: Instant,
    frames_in_window: u32,
}

impl Pacer {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            next_frame: now,
            window_started: now,
            frames_in_window: 0,
        }
    }

    /// Restart pacing after an idle period, so time spent paused is not
    /// mistaken for emulation debt.
    fn resume(&mut self) {
        *self = Self::new();
    }

    /// Sleep out the remainder of the current frame, returning a throughput
    /// report roughly once a second.
    fn frame_completed(&mut self) -> Option<CoreMessage> {
        self.frames_in_window += 1;
        self.next_frame += FRAME_TIME;

        let now = Instant::now();
        if let Some(remaining) = self.next_frame.checked_duration_since(now) {
            std::thread::sleep(remaining);
        } else if now.duration_since(self.next_frame) > FRAME_TIME * MAX_CATCHUP_FRAMES {
            // Hopelessly behind: drop the backlog and pace from here.
            self.next_frame = now;
        }

        let elapsed = now.duration_since(self.window_started);
        if elapsed < SPEED_REPORT_INTERVAL {
            return None;
        }

        let fps = f64::from(self.frames_in_window) / elapsed.as_secs_f64();
        self.window_started = now;
        self.frames_in_window = 0;

        Some(CoreMessage::Speed {
            fps: fps as f32,
            percent: (fps * FRAME_TIME.as_secs_f64() * 100.0) as f32,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Device, RAM_BANK_SIZE};
    use crate::{
        constants::{Interrupts, ROM_BANK_SIZE},
        mbc::Mbc,
        Address, Byte,
    };

    #[test]
    fn if_and_ie_are_independent_registers() {
        // They used to be one field, so writing one clobbered the other.
        let mut dev = Device::new();
        dev.write(Address(0xFF0F), Byte(0x1F));
        dev.write(Address(0xFFFF), Byte(0x00));

        assert_eq!(dev.read(Address(0xFF0F)).0 & 0x1F, 0x1F);
        assert_eq!(dev.read(Address(0xFFFF)).0, 0x00);
    }

    #[test]
    fn enabled_interrupt_is_dispatched_to_its_vector() {
        let mut dev = Device::new();
        dev.cpu.interupt_master_enable = true;
        dev.write(Address(0xFFFF), Byte(0x01)); // enable VBlank
        dev.request_interrupts(Interrupts::VBlank);

        dev.step_cpu();

        assert_eq!(dev.cpu.pc, Address(0x0040), "vectored to the VBlank handler");
        assert_eq!(dev.cpu.sp, Address(0xFFFC), "return address pushed");
        assert!(!dev.cpu.interupt_master_enable, "IME cleared on dispatch");
        assert_eq!(dev.read(Address(0xFF0F)).0 & 0x01, 0, "request acknowledged");
    }

    #[test]
    fn highest_priority_interrupt_wins() {
        let mut dev = Device::new();
        dev.cpu.interupt_master_enable = true;
        dev.write(Address(0xFFFF), Byte(0x1F)); // enable all
        dev.request_interrupts(Interrupts::Timer | Interrupts::Joypad);

        dev.step_cpu();

        // Timer (bit 2 -> 0x50) outranks Joypad (bit 4 -> 0x60).
        assert_eq!(dev.cpu.pc, Address(0x0050));
        assert_eq!(dev.read(Address(0xFF0F)).0 & 0x04, 0, "only Timer cleared");
        assert_eq!(dev.read(Address(0xFF0F)).0 & 0x10, 0x10, "Joypad still pending");
    }

    #[test]
    fn interrupts_are_ignored_while_ime_is_clear() {
        let mut dev = Device::new();
        dev.write(Address(0xFFFF), Byte(0x01));
        dev.request_interrupts(Interrupts::VBlank);

        dev.step_cpu();

        // No vectoring: the CPU just runs the next instruction normally, so the
        // request stays pending and nothing was pushed.
        assert_ne!(dev.cpu.pc, Address(0x0040), "must not vector without IME");
        assert_eq!(dev.cpu.sp, Address(0xFFFE), "nothing pushed");
        assert_eq!(dev.read(Address(0xFF0F)).0 & 0x01, 0x01, "request still pending");
    }

    #[test]
    fn halt_resumes_on_a_pending_interrupt_even_without_ime() {
        let mut dev = Device::new();
        dev.cpu.halted = true;
        dev.write(Address(0xFFFF), Byte(0x01));
        dev.request_interrupts(Interrupts::VBlank);

        dev.step_cpu();

        // HALT ends, but with IME clear the interrupt is not serviced: execution
        // simply resumes, leaving the request pending.
        assert!(!dev.cpu.halted, "HALT ends when an enabled interrupt is pending");
        assert_ne!(dev.cpu.pc, Address(0x0040), "not vectored (IME clear)");
        assert_eq!(dev.read(Address(0xFF0F)).0 & 0x01, 0x01, "request still pending");
    }

    #[test]
    fn switchable_bank_addresses_past_64k_without_wrapping() {
        // Four banks, each filled with its own bank number, so a byte's value
        // reveals which bank a read actually landed in.
        let mut dev = Device::new();
        dev.rom = (0..4 * ROM_BANK_SIZE)
            .map(|i| Byte((i / ROM_BANK_SIZE) as u8))
            .collect();
        dev.mbc = Mbc::new(0x01, 4, 0); // MBC1

        // Bank 0 is fixed at 0x0000-0x3FFF.
        assert_eq!(dev.read(Address(0x0000)), Byte(0));
        assert_eq!(dev.read(Address(0x3FFF)), Byte(0));

        // Selecting bank 3 puts physical offset 0xC000 in the window, which
        // overflowed the old u16 arithmetic to 0x0000.
        dev.write(Address(0x2000), Byte(3));
        assert_eq!(dev.read(Address(0x4000)), Byte(3));
        assert_eq!(dev.read(Address(0x7FFF)), Byte(3));
    }

    #[test]
    fn reads_past_the_rom_return_open_bus() {
        // A ROM shorter than an addressed offset (e.g. padding past a small
        // image) reads back as open bus rather than panicking.
        let mut dev = Device::new();
        dev.rom = vec![Byte(0x11); 4];
        assert_eq!(dev.read_rom(100), Byte(0xFF));
    }

    #[test]
    fn rom_writes_program_the_mbc_and_never_touch_the_image() {
        let mut dev = Device::new();
        dev.rom = vec![Byte(0xAB); 4 * ROM_BANK_SIZE];
        dev.mbc = Mbc::new(0x01, 4, 0);

        // A write into the ROM range is a bank select, not a store.
        dev.write(Address(0x2000), Byte(2));
        assert_eq!(dev.read(Address(0x0000)), Byte(0xAB), "ROM image untouched");
        // And it took effect: the window now maps bank 2.
        assert_eq!(dev.read(Address(0x4000)), Byte(0xAB));
    }

    #[test]
    fn external_ram_round_trips_only_while_enabled() {
        let mut dev = Device::new();
        dev.rom = vec![Byte(0); 4 * ROM_BANK_SIZE];
        dev.eram = vec![Byte(0); RAM_BANK_SIZE];
        dev.mbc = Mbc::new(0x03, 4, 1); // MBC1 + RAM

        // Disabled by default: writes drop, reads float high.
        dev.write(Address(0xA000), Byte(0x42));
        assert_eq!(dev.read(Address(0xA000)), Byte(0xFF));

        // Enable RAM (0x0A into 0x0000-0x1FFF), then it round-trips.
        dev.write(Address(0x0000), Byte(0x0A));
        dev.write(Address(0xA000), Byte(0x42));
        assert_eq!(dev.read(Address(0xA000)), Byte(0x42));
    }

    #[test]
    fn load_cartrige_loads_the_whole_file_including_vectors() {
        use std::io::Write;

        // A minimal 32 KB image with a valid-enough header (ROM ONLY, 32 KB) so
        // header parsing neither panics on get_rom_size nor transmutes a bad
        // MBC discriminant.
        let mut rom = vec![0u8; ROM_BANK_SIZE * 2];
        rom[0x0040] = 0xAB; // VBlank vector - was left as zero by the old loader
        rom[0x0147] = 0x00; // MBC type: ROM ONLY
        rom[0x0148] = 0x00; // ROM size: 32 KB
        rom[0x7FFF] = 0xCD; // last byte of the last bank

        let mut file = tempfile::NamedTempFile::new().expect("temp rom");
        file.write_all(&rom).expect("write rom");
        file.flush().expect("flush rom");

        let mut dev = Device::new();
        dev.load_cartrige(file.path()).expect("load");

        assert_eq!(dev.read(Address(0x0040)), Byte(0xAB), "vectors loaded");
        assert_eq!(dev.read(Address(0x7FFF)), Byte(0xCD), "whole file loaded");
        assert_eq!(
            dev.get_cartridge_header().expect("header").rom_size(),
            0x8000,
        );
    }

    #[test]
    fn cpu_advances_one_machine_cycle_per_four_ticks() {
        let mut dev = Device::new();
        dev.write(Address(0x0100), Byte(0x00)); // NOP at the reset vector

        // One NOP is one machine cycle: four master ticks execute exactly one.
        dev.tick(4);

        assert_eq!(dev.cpu.pc, Address(0x0101));
    }
}
