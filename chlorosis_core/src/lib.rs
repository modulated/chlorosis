#![deny(clippy::all)]
#![deny(clippy::nursery)]

mod audio;
mod cpu;
pub mod device;
pub mod framebuffer;
mod frontend;
mod infrared;
mod serial;
mod joypad;
mod mbc;
mod ppu;
pub mod savestate;
mod timer;
mod types;
pub use audio::{AudioProcessor, SAMPLE_RATE};
pub use cpu::CentralProcessor;
pub use device::{Device, EmulatorState, TICKS_PER_FRAME};
pub use framebuffer::{Frame, FrameConsumer, FrameProducer, SCREEN_HEIGHT, SCREEN_WIDTH};
pub use frontend::{
    channels, AudioBuffer, CoreChannels, CoreMessage, Event, FrontendChannels, KeyCode,
    AUDIO_BUFFER_CAP,
};
pub use infrared::Infrared;
pub use serial::Serial;
pub use joypad::Joypad;
pub use ppu::PixelProcessor;
pub use savestate::SaveStateError;
pub use timer::Timer;
pub(crate) use types::{constants, Address, Byte, SignedByte};
