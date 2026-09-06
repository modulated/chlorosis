//! The protocol between the frontend and the emulation thread.
//!
//! Everything crossing the thread boundary goes through here, in three
//! directions that are deliberately kept apart:
//!
//! * [`Event`] - intents from the frontend. The frontend never mutates the
//!   emulator, it asks.
//! * [`CoreMessage`] - what actually happened, reported back. The emulator owns
//!   its state; the frontend renders what it is told rather than keeping a
//!   second copy that can drift out of sync with the real one.
//! * frames, over the swap in [`crate::framebuffer`], which must not be allowed
//!   to queue up behind a slow frontend.

use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::{
    device::EmulatorState,
    framebuffer::{frame_channel, FrameConsumer, FrameProducer},
};

/// A request from the frontend to the emulation thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    KeyDown(Vec<KeyCode>),
    KeyUp(Vec<KeyCode>),
    LoadFile(PathBuf),
    SaveState(PathBuf),
    LoadState(PathBuf),
    Run,
    Pause,
    /// Advance the given number of master clock ticks while paused.
    Step(u32),
    Reset,
    /// Shut the emulation thread down so it can be joined.
    Exit,
}

/// A report from the emulation thread to the frontend.
#[derive(Debug, Clone, PartialEq)]
pub enum CoreMessage {
    /// The emulator changed state. This is the only authority on it.
    State(EmulatorState),
    CartridgeLoaded(String),
    /// Something the user asked for did not work, but emulation continues.
    Error(String),
    /// Emulation stopped dead - a panic or unimplemented hardware. Without
    /// this the frontend would sit in front of a frozen picture with no idea
    /// why the emulator went quiet.
    Faulted(String),
    /// Throughput over the last reporting interval, where `percent` is speed
    /// relative to real hardware.
    Speed { fps: f32, percent: f32 },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KeyCode {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    Start,
    Select,
}

/// The emulation thread's endpoints.
#[derive(Debug)]
pub struct CoreChannels {
    pub events: Receiver<Event>,
    pub messages: Sender<CoreMessage>,
    pub frames: FrameProducer,
}

/// The frontend's endpoints.
#[derive(Debug)]
pub struct FrontendChannels {
    pub events: Sender<Event>,
    pub messages: Receiver<CoreMessage>,
    pub frames: FrameConsumer,
}

/// Wire up a frontend and an emulation thread.
#[must_use]
pub fn channels() -> (CoreChannels, FrontendChannels) {
    let (event_tx, event_rx) = mpsc::channel();
    let (message_tx, message_rx) = mpsc::channel();
    let (frame_tx, frame_rx) = frame_channel();

    (
        CoreChannels {
            events: event_rx,
            messages: message_tx,
            frames: frame_tx,
        },
        FrontendChannels {
            events: event_tx,
            messages: message_rx,
            frames: frame_rx,
        },
    )
}
