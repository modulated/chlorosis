//! The emulation thread's contract with a frontend.
//!
//! These exercise the thread boundary rather than the hardware: that requests
//! are answered promptly, that a bad request is reported instead of taking the
//! thread down, and that emulation actually keeps up with real time.

use std::{
    io::Write,
    sync::mpsc::RecvTimeoutError,
    thread,
    time::{Duration, Instant},
};

use chlorosis_core::{channels, CoreMessage, Device, EmulatorState, Event, FrontendChannels};

/// Generous enough that a loaded machine will not trip it, short enough that a
/// core which is not listening still fails the test.
const TIMEOUT: Duration = Duration::from_secs(5);

struct Harness {
    frontend: FrontendChannels,
    core: Option<thread::JoinHandle<()>>,
}

impl Harness {
    fn start() -> Self {
        let (core_channels, frontend) = channels();
        let core = thread::Builder::new()
            .name("core".to_owned())
            .spawn(move || Device::default().run(core_channels))
            .expect("spawn core");

        Self {
            frontend,
            core: Some(core),
        }
    }

    fn send(&self, event: Event) {
        self.frontend.events.send(event).expect("core is listening");
    }

    /// The next message matching `wanted`, ignoring anything else the core
    /// happens to report on the way.
    fn expect<T>(&self, wanted: impl Fn(&CoreMessage) -> Option<T>) -> T {
        let deadline = Instant::now() + TIMEOUT;
        loop {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .expect("timed out waiting for the core");
            match self.frontend.messages.recv_timeout(remaining) {
                Ok(message) => {
                    if let Some(found) = wanted(&message) {
                        return found;
                    }
                }
                Err(RecvTimeoutError::Timeout) => panic!("timed out waiting for the core"),
                Err(RecvTimeoutError::Disconnected) => panic!("core hung up"),
            }
        }
    }

    fn shutdown(&mut self) {
        self.send(Event::Exit);
        let core = self.core.take().expect("not already shut down");

        // join() has no timeout, so poll for the thread finishing instead of
        // hanging the whole test run if the core ignores Exit.
        let deadline = Instant::now() + TIMEOUT;
        while !core.is_finished() {
            assert!(Instant::now() < deadline, "core ignored Exit");
            thread::sleep(Duration::from_millis(10));
        }
        core.join().expect("core did not panic");
    }
}

fn state(message: &CoreMessage) -> Option<EmulatorState> {
    match message {
        CoreMessage::State(state) => Some(*state),
        _ => None,
    }
}

fn error(message: &CoreMessage) -> Option<String> {
    match message {
        CoreMessage::Error(e) => Some(e.clone()),
        _ => None,
    }
}

/// A ROM that jumps to itself forever, so the CPU can be run for a measured
/// stretch without wandering into unimplemented hardware.
fn spin_rom() -> tempfile::NamedTempFile {
    let mut rom = vec![0u8; 0x8000];
    rom[0x0100] = 0xC3; // JP a16
    rom[0x0101] = 0x00;
    rom[0x0102] = 0x01; // -> 0x0100

    let mut file = tempfile::NamedTempFile::new().expect("temp rom");
    file.write_all(&rom).expect("write rom");
    file.flush().expect("flush rom");
    file
}

#[test]
fn exit_shuts_the_core_down_while_it_is_idle() {
    // A core that only polls for events between frames leaves the frontend
    // waiting on shutdown; one that blocks on the channel answers immediately.
    let mut harness = Harness::start();
    harness.shutdown();
}

#[test]
fn a_missing_rom_is_reported_rather_than_fatal() {
    let mut harness = Harness::start();

    harness.send(Event::LoadFile("/nonexistent/cartrige.gbc".into()));
    let reported = harness.expect(error);
    assert!(reported.contains("cartrige.gbc"), "{reported}");

    // The core is still alive and still listening.
    harness.shutdown();
}

#[test]
fn running_without_a_cartrige_is_reported_rather_than_fatal() {
    let mut harness = Harness::start();

    harness.send(Event::Run);
    assert!(harness.expect(error).contains("cartrige"));

    harness.shutdown();
}

#[test]
fn loading_a_cartrige_starts_emulation_and_reports_it() {
    let rom = spin_rom();
    let mut harness = Harness::start();

    harness.send(Event::LoadFile(rom.path().to_path_buf()));
    assert_eq!(harness.expect(state), EmulatorState::Running);

    harness.send(Event::Pause);
    assert_eq!(harness.expect(state), EmulatorState::Paused);

    harness.shutdown();
}

#[test]
fn emulation_keeps_up_with_real_time() {
    // The whole point of the split: the core must reach roughly real hardware
    // speed on its own thread. Sleeping per tick instead of per frame lands
    // orders of magnitude below this, so a wide tolerance still catches it.
    let rom = spin_rom();
    let mut harness = Harness::start();

    harness.send(Event::LoadFile(rom.path().to_path_buf()));

    let percent = harness.expect(|m| match m {
        CoreMessage::Speed { percent, .. } => Some(*percent),
        _ => None,
    });

    assert!(
        (25.0..=150.0).contains(&percent),
        "emulated speed was {percent}% of real hardware"
    );

    harness.shutdown();
}
