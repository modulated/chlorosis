#![deny(clippy::all)]
#![deny(clippy::nursery)]
// #![deny(clippy::pedantic)]
#![allow(dead_code)] // TODO: Remove later

mod emu;
pub use emu::device::Device;
