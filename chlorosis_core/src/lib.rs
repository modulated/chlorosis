#![deny(clippy::all)]
#![deny(clippy::nursery)]

mod audio;
mod cpu;
pub mod device;
mod frontend;
mod infrared;
mod joypad;
mod ppu;
mod timer;
mod types;
pub use audio::AudioProcessor;
pub use cpu::CentralProcessor;
pub use device::Device;
pub use frontend::{Event, Frontend, KeyCode};
pub use infrared::Infrared;
pub use joypad::Joypad;
pub use ppu::PixelProcessor;
pub use timer::Timer;
pub(crate) use types::{constants, Address, Byte, SignedByte};
