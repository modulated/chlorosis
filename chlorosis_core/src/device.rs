use std::{
    any::Any,
    panic::{catch_unwind, AssertUnwindSafe},
    path::PathBuf,
    sync::mpsc::TryRecvError,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

use super::{Address, Byte};

use crate::{
    constants::*, mbc::Mbc, savestate::{self, SaveStateError}, CoreChannels, CoreMessage, Event,
    Infrared, Joypad, KeyCode, Serial, Timer,
};

use super::{types::CartrigeHeader, AudioProcessor, CentralProcessor, PixelProcessor};

/// Master clock ticks in one video frame: 154 lines of 456 dots.
pub const TICKS_PER_FRAME: u32 = 70_224;

/// 4.194304 MHz / 70224 ticks == 59.7275 Hz.
const FRAME_TIME: Duration = Duration::from_nanos(16_742_706);

/// External-RAM bank size (8 KB), used to size `eram` and count RAM banks.
const RAM_BANK_SIZE: usize = 0x2000;

/// How far behind real time the emulator may fall before it stops trying to
/// catch up. Without this a long host stall (a dragged window, a swapped out
/// page) leaves a debt the emulator repays by sprinting through several frames
/// of gameplay at once.
const MAX_CATCHUP_FRAMES: u32 = 4;

/// How often throughput is reported to the frontend.
const SPEED_REPORT_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub cpu: CentralProcessor,
    ppu: PixelProcessor,
    audio: AudioProcessor,
    // The cartridge header, the ROM image, and the ROM's path are not part of a
    // save state: the ROM is immutable and reloaded from disk, and the header is
    // derived from it. They are kept from the live machine across a load rather
    // than serialized.
    #[serde(skip)]
    cartrige: Option<CartrigeHeader>,
    joypad: Joypad,
    #[serde(skip)]
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
    serial: Serial,
    timer: Timer,
    state: EmulatorState,
    #[serde(skip)]
    rom_path: Option<PathBuf>,
}

/// Whether the emulator is executing. The emulation thread is the only owner of
/// this; the frontend learns about changes through [`CoreMessage::State`]
/// rather than tracking its own copy.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
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
            audio: AudioProcessor::default(),
            cartrige: None,
            joypad: Joypad::default(),
            infrared: Infrared::default(),
            serial: Serial::default(),
            timer: Timer::default(),
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
        let result = catch_unwind(AssertUnwindSafe(|| self.run_loop(&channels)));

        // Persist battery-backed RAM on the way out - a clean shutdown or a
        // fault both end here, so progress is saved either way.
        self.flush_battery();

        if let Err(panic) = result {
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
        self.publish_audio(channels);

        if let Some(report) = pacer.frame_completed()
            && channels.messages.send(report).is_err()
        {
            return Control::Shutdown;
        }

        Control::Continue
    }

    /// Advance every component by `ticks` of the master clock. Public so a
    /// headless harness can drive the machine directly, without the frame pacer
    /// or the frontend channels that [`Self::run`] uses.
    pub fn tick(&mut self, ticks: u32) {
        for _ in 0..ticks {
            // The PPU, timer, and APU run on the master clock, one step per tick.
            let mut pending = self.ppu.step();
            if self.timer.tick() {
                pending |= Interrupts::Timer;
            }
            self.audio.tick();
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

    /// Whether any interrupt is both requested (IF) and enabled (IE). This is
    /// what ends HALT and, with IME clear at a `HALT`, triggers the HALT bug.
    pub(crate) const fn has_pending_interrupt(&self) -> bool {
        self.interrupt_flag.0 & self.interrupt_enable.0 & 0x1F != 0
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
            frame.copy_from_slice(&rendered[..]);
            channels.frames.publish(frame);
        }
    }

    /// Hand the frontend the audio samples the APU produced this frame. The
    /// shared buffer is capped, so if nothing is draining it (audio disabled, or
    /// the machine paused) the excess is dropped rather than growing without
    /// bound.
    fn publish_audio(&mut self, channels: &CoreChannels) {
        let samples = self.audio.drain();
        if samples.is_empty() {
            return;
        }
        if let Ok(mut buffer) = channels.audio.lock() {
            let room = crate::frontend::AUDIO_BUFFER_CAP.saturating_sub(buffer.len());
            buffer.extend(samples.into_iter().take(room));
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
            Event::SaveState(path) => self.handle_save_state(path, channels),
            Event::LoadState(path) => self.handle_load_state(path, channels),
            Event::QuickSave(slot) => match self.slot_path(slot) {
                Some(path) => self.handle_save_state(path, channels),
                None => report(
                    channels,
                    CoreMessage::Error("No ROM loaded to quick-save".to_owned()),
                ),
            },
            Event::QuickLoad(slot) => match self.slot_path(slot) {
                Some(path) => self.handle_load_state(path, channels),
                None => report(
                    channels,
                    CoreMessage::Error("No ROM loaded to quick-load".to_owned()),
                ),
            },
            Event::Exit => return Control::Shutdown,
        }
        Control::Continue
    }

    fn handle_load_file(&mut self, path: &std::path::Path, channels: &CoreChannels) {
        // Save the outgoing cartridge's RAM before its ROM/path is replaced.
        self.flush_battery();
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

    fn handle_save_state(&self, path: PathBuf, channels: &CoreChannels) {
        match self.save_state_to(&path) {
            Ok(()) => report(
                channels,
                CoreMessage::Notice(format!("Saved state to {}", path.display())),
            ),
            Err(e) => report(
                channels,
                CoreMessage::Error(format!("Could not save state to {}: {e}", path.display())),
            ),
        }
    }

    fn handle_load_state(&mut self, path: PathBuf, channels: &CoreChannels) {
        match self.load_state_from(&path) {
            Ok(()) => {
                report(
                    channels,
                    CoreMessage::Notice(format!("Loaded state from {}", path.display())),
                );
                // The restored state carries its own running/paused flag; the
                // frontend tracks a copy, so tell it what the machine is now.
                report(channels, CoreMessage::State(self.state));
            }
            Err(e) => report(
                channels,
                CoreMessage::Error(format!("Could not load state from {}: {e}", path.display())),
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

    /// Restore power on state, keeping whatever cartridge is inserted. Battery
    /// RAM is flushed first and reloaded when the cartridge loads again, so a
    /// reset persists save data the way a real power cycle does.
    fn reset(&mut self) -> Result<(), std::io::Error> {
        self.flush_battery();
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

        // A CGB cartridge drives the colour renderer; a plain DMG cart keeps the
        // greyscale path (BGP/OBP shades) it writes.
        self.ppu.set_cgb_mode(header.is_cgb());

        self.cartrige = Some(header);
        self.rom_path = Some(path.to_path_buf());

        // Restore battery-backed save RAM for this cartridge, if any.
        self.load_battery();

        self.dump_cartrige_header();

        Ok(())
    }

    /// Serialize the whole machine (minus the ROM, which is reloaded on restore)
    /// into a save-state blob: the header (magic, format version, ROM checksum)
    /// followed by the `bincode` payload.
    pub fn save_state(&self) -> Result<Vec<u8>, SaveStateError> {
        let header = self.cartrige.as_ref().ok_or(SaveStateError::NoCartridge)?;

        let mut out = Vec::with_capacity(savestate::HEADER_LEN + 0x10000);
        out.extend_from_slice(&savestate::MAGIC);
        out.extend_from_slice(&savestate::FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&header.global_checksum().to_le_bytes());
        bincode::serialize_into(&mut out, self)?;
        Ok(out)
    }

    /// Restore the machine from a blob produced by [`Self::save_state`], keeping
    /// the currently loaded ROM. Validates the magic, format version, and ROM
    /// checksum before touching any state, so a bad or foreign file leaves the
    /// running machine untouched.
    pub fn load_state(&mut self, data: &[u8]) -> Result<(), SaveStateError> {
        let header = self.cartrige.as_ref().ok_or(SaveStateError::NoCartridge)?;

        if data.len() < savestate::HEADER_LEN {
            return Err(SaveStateError::Truncated);
        }
        if data[0..4] != savestate::MAGIC {
            return Err(SaveStateError::NotASaveState);
        }
        let version = u16::from_le_bytes([data[4], data[5]]);
        if version != savestate::FORMAT_VERSION {
            return Err(SaveStateError::VersionMismatch {
                found: version,
                expected: savestate::FORMAT_VERSION,
            });
        }
        let checksum = u16::from_le_bytes([data[6], data[7]]);
        if checksum != header.global_checksum() {
            return Err(SaveStateError::WrongRom {
                found: checksum,
                expected: header.global_checksum(),
            });
        }

        // Decode into a fresh machine first: if it fails, `self` is untouched.
        let mut restored: Self = bincode::deserialize(&data[savestate::HEADER_LEN..])?;

        // The ROM, its path, and the header are not in the payload; carry the
        // live ones across so the restored machine keeps running this game.
        restored.rom = std::mem::take(&mut self.rom);
        restored.rom_path = self.rom_path.take();
        restored.cartrige = self.cartrige.take();
        *self = restored;
        Ok(())
    }

    /// Write a save state to `path`.
    pub fn save_state_to(&self, path: impl AsRef<std::path::Path>) -> Result<(), SaveStateError> {
        let bytes = self.save_state()?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Read and restore a save state from `path`.
    pub fn load_state_from(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), SaveStateError> {
        let bytes = std::fs::read(path)?;
        self.load_state(&bytes)
    }

    /// The conventional path for quick-save slot `slot`, alongside the ROM:
    /// `<rom>.<slot>.chl`. `None` when no ROM path is known.
    pub fn slot_path(&self, slot: u8) -> Option<PathBuf> {
        self.rom_path.as_ref().map(|rom| {
            let mut name = rom.as_os_str().to_owned();
            name.push(format!(".{slot}.chl"));
            PathBuf::from(name)
        })
    }

    /// The battery-backed save-RAM file for the current ROM: `<rom>.sav`.
    fn battery_path(&self) -> Option<PathBuf> {
        self.rom_path.as_ref().map(|rom| {
            let mut name = rom.as_os_str().to_owned();
            name.push(".sav");
            PathBuf::from(name)
        })
    }

    /// Whether the loaded cartridge type (header `0x0147`) has battery-backed
    /// RAM, i.e. RAM that persists with the power off and so is worth a `.sav`.
    fn has_battery(&self) -> bool {
        matches!(
            self.rom.get(0x0147).map_or(0, |b| b.0),
            // MBC1/2/3/5 +RAM+BATTERY and the battery-backed ROM/MMM01/HuC types.
            0x03 | 0x06 | 0x09 | 0x0D | 0x0F | 0x10 | 0x13 | 0x1B | 0x1E | 0x22 | 0xFF
        )
    }

    /// Load `<rom>.sav` into external RAM if this cart has a battery and the file
    /// exists and matches the RAM size. A missing file (a brand-new save) or a
    /// mismatched one is ignored, so the game just starts with blank RAM.
    fn load_battery(&mut self) {
        if !self.has_battery() || self.eram.is_empty() {
            return;
        }
        let Some(path) = self.battery_path() else {
            return;
        };
        match std::fs::read(&path) {
            Ok(bytes) if bytes.len() == self.eram.len() => {
                self.eram = bytes.into_iter().map(Byte).collect();
                println!("Loaded battery RAM from {}", path.display());
            }
            Ok(bytes) => eprintln!(
                "Ignoring {}: {} bytes of save RAM, expected {}",
                path.display(),
                bytes.len(),
                self.eram.len()
            ),
            // No file yet is the normal case for a fresh cartridge.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => eprintln!("Could not read {}: {e}", path.display()),
        }
    }

    /// Write external RAM out to `<rom>.sav` if this cart has a battery. Called
    /// on shutdown and before swapping or resetting a cartridge, so progress
    /// survives across runs. Errors are logged, never fatal.
    pub fn flush_battery(&self) {
        if !self.has_battery() || self.eram.is_empty() {
            return;
        }
        let Some(path) = self.battery_path() else {
            return;
        };
        let bytes: Vec<u8> = self.eram.iter().map(|b| b.0).collect();
        match std::fs::write(&path, bytes) {
            Ok(()) => println!("Saved battery RAM to {}", path.display()),
            Err(e) => eprintln!("Could not write {}: {e}", path.display()),
        }
    }

    /// Bytes the ROM has shifted out of the serial port - its console output,
    /// used by test ROMs to report results.
    pub fn read_dbg(&mut self, x: u16) -> u8 { self.read(Address(x)).0 }

    pub fn serial_output(&self) -> &[u8] {
        self.serial.output()
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

    /// Physical `wram` index for a `0xD000-0xDFFF` access.
    ///
    /// The bank register selecting 0 means bank 1 on hardware, both on DMG and
    /// CGB. Without that, a `0xD000-0xDFFF` access with the register at 0 mapped
    /// onto the same bytes as `0xC000-0xCFFF`, so a write there silently
    /// corrupted whatever lived in the fixed bank - including code.
    const fn wram_1_index(&self, address: Address) -> usize {
        let bank = if self.wram_bank.0 == 0 { 1 } else { self.wram_bank.0 } as usize;
        (address.0 as usize - WRAM_1_START as usize) + bank * WRAM_BANK_SIZE
    }

    /// Copy the 160-byte OAM block from the page `page << 8` into the PPU's OAM,
    /// the effect of a write to the DMA register (`0xFF46`).
    fn oam_dma(&mut self, page: Byte) {
        let source = u16::from(page.0) << 8;
        for i in 0..OAM_SIZE as u16 {
            let byte = self.read(Address(source + i));
            self.ppu.oam[i as usize] = byte;
        }
    }

    /// Perform a CGB VRAM DMA (the `0xFF55` register). Both the general-purpose
    /// and HBlank variants are run in full immediately; the HBlank version is
    /// not spread across scanlines, which is enough to get tiles into VRAM even
    /// if it is not cycle accurate. This is what CGB games use to load most of
    /// their graphics, so without it the screen stays garbled.
    fn vram_dma(&mut self) {
        let (source, dest, length) = self.ppu.hdma_params();
        for i in 0..length as u16 {
            let byte = self.read(Address(source.wrapping_add(i)));
            self.ppu.dma_write_vram(Address(dest.wrapping_add(i)), byte);
        }
        self.ppu.hdma_finish();
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
            WRAM_1_START..=WRAM_1_END => self.wram[self.wram_1_index(address)],
            // Echo RAM mirrors WRAM 0x2000 below it.
            DEADZONE_0_START..=DEADZONE_0_END => self.read(address - Address(0x2000)),
            OAM_START..=OAM_END => self.ppu.read_oam(address),
            DEADZONE_1_START..=DEADZONE_1_END => Byte(0xFF), // unusable region

            // IO START
            0xFF00 => self.joypad.read(), // Joypad
            0xFF01..=0xFF02 => self.serial.read(address), // Serial
            0xFF04..=0xFF07 => self.timer.read(address), // Timers
            0xFF0F => Byte(self.interrupt_flag.0 | 0xE0), // IF (top 3 bits read as 1)
            0xFF10..=0xFF3F => self.audio.read(address), // Audio
            0xFF40..=0xFF55 => self.ppu.read_io(address), // PPU
            0xFF56 => self.infrared.read(), // Infrared Com Port
            0xFF57..=0xFF6F => self.ppu.read_io(address), // PPU
            0xFF70 => self.wram_bank,     // WRAM BANK
            0xFF76 => self.audio.read(address), // PCM12 (read-only)
            0xFF77 => self.audio.read(address), // PCM34 (read-only)
            // Prohibited and unused IO holes read as open bus rather than
            // faulting the core - a stray access must never take the thread down.
            0xFF03 | 0xFF08..=0xFF0E | 0xFF71..=0xFF75 | 0xFF78..=0xFF7F => Byte(0xFF),
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
                if let Some(offset) = self.mbc.ram_offset(address.0)
                    && let Some(cell) = self.eram.get_mut(offset)
                {
                    *cell = value;
                }
            }
            WRAM_0_START..=WRAM_0_END => self.wram[address - Address(WRAM_0_START)] = value,
            WRAM_1_START..=WRAM_1_END => {
                let index = self.wram_1_index(address);
                self.wram[index] = value;
            }
            // Echo RAM mirrors WRAM 0x2000 below it.
            DEADZONE_0_START..=DEADZONE_0_END => self.write(address - Address(0x2000), value),
            OAM_START..=OAM_END => self.ppu.write_oam(address, value),
            DEADZONE_1_START..=DEADZONE_1_END => {} // unusable region

            // IO_START
            0xFF00 => {
                // Selecting a group that holds a pressed button also drops a
                // line and raises the Joypad interrupt.
                if self.joypad.write(value) {
                    self.request_interrupts(Interrupts::Joypad);
                }
            }
            0xFF01..=0xFF02 => self.serial.write(address, value), // Serial
            0xFF04..=0xFF07 => self.timer.write(address, value), // Timers
            0xFF0F => self.interrupt_flag = Byte(value.0 & 0x1F), // IF
            0xFF10..=0xFF3F => self.audio.write(address, value), // Audio
            0xFF46 => {
                // OAM DMA: copy 0xA0 bytes from XX00 into OAM. Real hardware
                // takes 160 machine cycles and locks the bus; this does it at
                // once, which is enough for sprites to appear.
                self.oam_dma(value);
                self.ppu.write_io(address, value);
            }
            0xFF55 => {
                // CGB VRAM DMA. Store the length/mode, then run the transfer.
                self.ppu.write_io(address, value);
                self.vram_dma();
            }
            0xFF40..=0xFF55 => self.ppu.write_io(address, value), // PPU
            0xFF56 => self.infrared.write(value), // Infrared Com Port
            0xFF57..=0xFF6F => self.ppu.write_io(address, value), // PPU
            0xFF70 => self.wram_bank = value,   // WRAM BANK
            0xFF76..=0xFF77 => {}               // Audio PCM12/34 (read-only)
            // Prohibited and unused IO holes drop writes rather than faulting.
            0xFF03 | 0xFF08..=0xFF0E | 0xFF71..=0xFF75 | 0xFF78..=0xFF7F => {}
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
        let mut edge = false;
        for b in keys {
            edge |= self.joypad.press(b);
        }
        // A button in a selected group falling from high to low raises the
        // Joypad interrupt, which is also what wakes the CPU from STOP.
        if edge {
            self.request_interrupts(Interrupts::Joypad);
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
    fn wram_bank_zero_does_not_alias_the_fixed_bank() {
        let mut dev = Device::new();
        dev.write(Address(0xC800), Byte(0xAA)); // fixed bank 0

        // Select WRAM bank 0; hardware treats it as bank 1, so 0xD000-0xDFFF
        // must not land on the same bytes as 0xC000-0xCFFF.
        dev.write(Address(0xFF70), Byte(0x00));
        dev.write(Address(0xD800), Byte(0xBB));

        assert_eq!(dev.read(Address(0xC800)), Byte(0xAA), "fixed bank corrupted");
        assert_eq!(dev.read(Address(0xD800)), Byte(0xBB));
    }

    #[test]
    fn oam_dma_copies_a_page_into_oam() {
        let mut dev = Device::new();
        // Fill a WRAM page with a recognizable pattern, then DMA it in.
        for i in 0..0xA0u16 {
            dev.write(Address(0xC000 + i), Byte((i as u8) ^ 0x5A));
        }
        dev.write(Address(0xFF46), Byte(0xC0)); // source 0xC000

        for i in 0..0xA0usize {
            assert_eq!(dev.ppu.oam[i], Byte((i as u8) ^ 0x5A), "OAM byte {i}");
        }
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

    /// Diagnostic: run the ROM at `CHLOROSIS_DUMP_ROM` for a number of frames and
    /// write the last rendered frame to `CHLOROSIS_DUMP_PPM` (default
    /// `frame.ppm`) as a P6 PPM. Skips unless the env var is set.
    #[test]
    fn dump_rom_frame() {
        use crate::{TICKS_PER_FRAME, SCREEN_HEIGHT, SCREEN_WIDTH};
        let Ok(path) = std::env::var("CHLOROSIS_DUMP_ROM") else {
            eprintln!("skipping: set CHLOROSIS_DUMP_ROM to render a frame");
            return;
        };
        let frames: u32 = std::env::var("CHLOROSIS_DUMP_FRAMES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(600);

        let mut dev = Device::new();
        dev.load_cartrige(&path).expect("load cartridge");

        let mut last = Box::new([0u32; SCREEN_WIDTH * SCREEN_HEIGHT]);
        for _ in 0..frames {
            dev.tick(TICKS_PER_FRAME);
            if let Some(frame) = dev.ppu.take_frame() {
                last = frame;
            }
        }

        let out = std::env::var("CHLOROSIS_DUMP_PPM").unwrap_or_else(|_| "frame.ppm".into());
        let mut buf = format!("P6\n{SCREEN_WIDTH} {SCREEN_HEIGHT}\n255\n").into_bytes();
        for &px in last.iter() {
            buf.push((px >> 16) as u8);
            buf.push((px >> 8) as u8);
            buf.push(px as u8);
        }
        std::fs::write(&out, buf).expect("write ppm");
        eprintln!("wrote {out}");
    }

    /// Diagnostic: prove a save state restores a real ROM exactly. Runs the ROM
    /// at `CHLOROSIS_DUMP_ROM` for a while, saves, runs on for a few more frames,
    /// then loads the state into a fresh machine and checks the next rendered
    /// frame matches the original's at that point. Skips unless the var is set.
    #[test]
    fn save_load_restores_a_real_rom() {
        use crate::TICKS_PER_FRAME;
        let Ok(path) = std::env::var("CHLOROSIS_DUMP_ROM") else {
            eprintln!("skipping: set CHLOROSIS_DUMP_ROM to exercise a real ROM");
            return;
        };

        let mut a = Device::new();
        a.load_cartrige(&path).expect("load");
        for _ in 0..200 {
            a.tick(TICKS_PER_FRAME);
        }
        let snapshot = a.save_state().expect("save");

        // Fresh machine, same ROM, restore the snapshot. Re-saving must be
        // byte-identical: everything serialized round-trips.
        let mut b = Device::new();
        b.load_cartrige(&path).expect("load");
        b.load_state(&snapshot).expect("restore");
        assert_eq!(snapshot, b.save_state().expect("re-save"), "state not faithful");

        // And the two machines must stay in lock-step: run both several frames
        // and compare the final rendered frame. A few frames lets the skipped
        // frame buffer refill, and surfaces any unsaved state that would make
        // them drift apart.
        let mut reference = a;
        let mut expected = None;
        let mut got = None;
        for _ in 0..4 {
            reference.tick(TICKS_PER_FRAME);
            b.tick(TICKS_PER_FRAME);
            expected = reference.ppu.take_frame();
            got = b.ppu.take_frame();
        }
        assert_eq!(expected, got, "restored machine drifted from the original");
    }

    /// A `Device` with a minimal ROM loaded, whose header global checksum is
    /// `checksum`. The temp file must be kept alive for `rom_path` to stay valid.
    fn device_with_rom(checksum: u16) -> (Device, tempfile::NamedTempFile) {
        use std::io::Write;
        let mut rom = vec![0u8; ROM_BANK_SIZE * 2];
        rom[0x0147] = 0x00; // MBC type: ROM ONLY
        rom[0x0148] = 0x00; // ROM size: 32 KB
        rom[0x014E] = (checksum >> 8) as u8; // global checksum, big-endian in header
        rom[0x014F] = checksum as u8;

        let mut file = tempfile::NamedTempFile::new().expect("temp rom");
        file.write_all(&rom).expect("write rom");
        file.flush().expect("flush rom");

        let mut dev = Device::new();
        dev.load_cartrige(file.path()).expect("load");
        (dev, file)
    }

    #[test]
    fn save_state_round_trips_the_machine() {
        let (mut dev, _rom) = device_with_rom(0x1234);

        // Put some recognizable state across several components.
        dev.cpu.a = Byte(0x42);
        dev.cpu.pc = Address(0x2468);
        dev.write(Address(0xC005), Byte(0x99)); // WRAM
        dev.ppu.vram[10] = Byte(0x77);
        dev.ppu.bcram[3] = Byte(0x5A);

        let saved = dev.save_state().expect("save");

        // Clobber all of it, then restore.
        dev.cpu.a = Byte(0);
        dev.cpu.pc = Address(0);
        dev.write(Address(0xC005), Byte(0));
        dev.ppu.vram[10] = Byte(0);
        dev.ppu.bcram[3] = Byte(0);

        dev.load_state(&saved).expect("load");

        assert_eq!(dev.cpu.a, Byte(0x42));
        assert_eq!(dev.cpu.pc, Address(0x2468));
        assert_eq!(dev.read(Address(0xC005)), Byte(0x99));
        assert_eq!(dev.ppu.vram[10], Byte(0x77));
        assert_eq!(dev.ppu.bcram[3], Byte(0x5A));
        // The ROM survived the restore (it is not part of the payload).
        assert!(dev.cartrige.is_some(), "cartridge kept across load");
    }

    #[test]
    fn save_state_without_a_cartridge_is_an_error() {
        let dev = Device::new();
        assert!(matches!(
            dev.save_state(),
            Err(crate::SaveStateError::NoCartridge)
        ));
    }

    #[test]
    fn load_state_rejects_a_non_save_file() {
        let (mut dev, _rom) = device_with_rom(0x1234);
        let err = dev.load_state(b"not a chlorosis save at all").unwrap_err();
        assert!(matches!(err, crate::SaveStateError::NotASaveState));
    }

    #[test]
    fn load_state_rejects_a_state_from_another_rom() {
        let (dev_a, _rom_a) = device_with_rom(0xAAAA);
        let saved = dev_a.save_state().expect("save");

        // A different ROM (different checksum) must refuse the state.
        let (mut dev_b, _rom_b) = device_with_rom(0xBBBB);
        let err = dev_b.load_state(&saved).unwrap_err();
        assert!(
            matches!(err, crate::SaveStateError::WrongRom { .. }),
            "got {err:?}"
        );
    }

    #[test]
    fn load_state_rejects_a_truncated_file() {
        let (mut dev, _rom) = device_with_rom(0x1234);
        let err = dev.load_state(&[0x01, 0x02]).unwrap_err();
        assert!(matches!(err, crate::SaveStateError::Truncated));
    }

    #[test]
    fn slot_path_sits_next_to_the_rom() {
        let (dev, rom) = device_with_rom(0x1234);
        let slot = dev.slot_path(0).expect("rom path known");
        let expected = format!("{}.0.chl", rom.path().display());
        assert_eq!(slot.to_string_lossy(), expected);
    }

    /// A `Device` with a minimal battery-backed MBC1 (+RAM+BATTERY) ROM loaded,
    /// sized for one 8 KB RAM bank.
    fn device_with_battery_rom() -> (Device, tempfile::NamedTempFile) {
        use std::io::Write;
        let mut rom = vec![0u8; ROM_BANK_SIZE * 2];
        rom[0x0147] = 0x03; // MBC1 + RAM + BATTERY
        rom[0x0148] = 0x00; // ROM size: 32 KB
        rom[0x0149] = 0x02; // RAM size: 8 KB

        let mut file = tempfile::NamedTempFile::new().expect("temp rom");
        file.write_all(&rom).expect("write rom");
        file.flush().expect("flush rom");

        let mut dev = Device::new();
        dev.load_cartrige(file.path()).expect("load");
        (dev, file)
    }

    #[test]
    fn battery_ram_detection() {
        let (battery, _b) = device_with_battery_rom();
        assert!(battery.has_battery(), "MBC1+RAM+BATTERY has a battery");

        let (plain, _p) = device_with_rom(0x1234); // ROM ONLY (0x00)
        assert!(!plain.has_battery(), "ROM ONLY has no battery");
    }

    #[test]
    fn battery_ram_persists_across_a_reload() {
        let (mut dev, rom) = device_with_battery_rom();
        let sav = dev.battery_path().expect("rom path known");

        // Enable RAM and write a recognizable byte, then flush to the .sav.
        dev.write(Address(0x0000), Byte(0x0A)); // RAM enable
        dev.write(Address(0xA042), Byte(0x77));
        dev.flush_battery();
        assert!(sav.exists(), "a .sav was written");

        // A fresh machine loading the same ROM restores that RAM.
        let mut fresh = Device::new();
        fresh.load_cartrige(rom.path()).expect("load");
        fresh.write(Address(0x0000), Byte(0x0A)); // RAM enable
        assert_eq!(fresh.read(Address(0xA042)), Byte(0x77), "save RAM restored");

        std::fs::remove_file(&sav).ok();
    }

    #[test]
    fn pressing_a_selected_button_requests_the_joypad_interrupt() {
        use crate::KeyCode;
        let mut dev = Device::new();
        dev.write(Address(0xFF00), Byte(0b0001_0000)); // select the action group

        dev.handle_keydown(vec![KeyCode::A]);
        assert_eq!(
            dev.read(Address(0xFF0F)).0 & Interrupts::Joypad.bits(),
            Interrupts::Joypad.bits(),
            "a selected press raises the Joypad interrupt",
        );
    }

    #[test]
    fn pressing_an_unselected_button_raises_no_interrupt() {
        use crate::KeyCode;
        let mut dev = Device::new();
        dev.write(Address(0xFF00), Byte(0b0001_0000)); // action group selected

        // A direction press is not on a selected line: no interrupt.
        dev.handle_keydown(vec![KeyCode::Up]);
        assert_eq!(dev.read(Address(0xFF0F)).0 & Interrupts::Joypad.bits(), 0);
    }

    #[test]
    fn halt_bug_executes_the_following_instruction_twice() {
        let mut dev = Device::new();
        // HALT, INC A, then NOPs, in writable WRAM.
        dev.cpu.pc = Address(0xC000);
        dev.write(Address(0xC000), Byte(0x76)); // HALT
        dev.write(Address(0xC001), Byte(0x3C)); // INC A
        dev.write(Address(0xC002), Byte(0x00)); // NOP
        dev.cpu.a = Byte(0);

        // The bug condition: IME clear with an interrupt already pending.
        dev.cpu.interupt_master_enable = false;
        dev.write(Address(0xFFFF), Byte(0x01)); // enable VBlank
        dev.request_interrupts(Interrupts::VBlank);

        // Step enough machine cycles to run HALT and reach the NOP.
        for _ in 0..12 {
            dev.step_cpu();
        }

        assert_eq!(dev.cpu.a, Byte(2), "INC A ran twice from the doubled fetch");
        assert!(!dev.cpu.halted, "the HALT bug does not halt the CPU");
    }

    #[test]
    fn halt_without_the_bug_condition_halts_normally() {
        let mut dev = Device::new();
        dev.cpu.pc = Address(0xC000);
        dev.write(Address(0xC000), Byte(0x76)); // HALT
        dev.cpu.a = Byte(0);
        // IME clear but nothing pending: a plain HALT, no bug.
        dev.cpu.interupt_master_enable = false;

        dev.step_cpu(); // fetch + execute HALT
        dev.step_cpu(); // consume its one cycle
        dev.step_cpu(); // at the boundary: still halted, nothing pending

        assert!(dev.cpu.halted, "HALT idles the CPU when no interrupt is pending");
        assert!(!dev.cpu.halt_bug);
    }

    #[test]
    fn prohibited_and_unused_io_never_faults() {
        let mut dev = Device::new();
        // These used to `panic!` or hit an `unreachable!()`, taking the core
        // thread down. They must now read open bus and drop writes instead.
        for addr in [0xFF03u16, 0xFF08, 0xFF0E, 0xFF4C, 0xFF4E, 0xFF50, 0xFF72, 0xFF7F] {
            dev.write(Address(addr), Byte(0x42)); // must not panic
            assert_eq!(dev.read(Address(addr)), Byte(0xFF), "open bus at {addr:#06X}");
        }
    }

    #[test]
    fn a_romless_cartridge_writes_no_sav() {
        // ROM ONLY: flush_battery must be a no-op, leaving no file behind.
        let (mut dev, _rom) = device_with_rom(0x1234);
        dev.write(Address(0x0000), Byte(0x0A));
        dev.write(Address(0xA000), Byte(0x55));
        dev.flush_battery();
        assert!(
            !dev.battery_path().expect("rom path").exists(),
            "no .sav for a battery-less cart"
        );
    }
}
