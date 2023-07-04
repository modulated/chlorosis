#![deny(clippy::all)]
#![deny(clippy::nursery)]
// #![deny(clippy::pedantic)]
// #![allow(dead_code)] // TODO: Remove later

mod audio;
mod cpu;
pub mod device;
mod frontend;
mod joypad;
mod ppu;
mod types;
pub use audio::AudioProcessor;
pub use cpu::CentralProcessor;
pub use device::Device;
pub use frontend::{Event, Frontend, KeyCode};
pub use joypad::Joypad;
pub use ppu::PixelProcessor;
pub(crate) use types::{constants, Address, Byte, SignedByte};
