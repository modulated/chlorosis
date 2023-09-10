use std::{io, path::PathBuf};

use crate::types::{Address, Byte};

mod mbc1;
mod mbc2;
mod mbc3;
mod mbc5;

trait Memory {
    fn from_bytes(bytes: Vec<u8>) -> Self;
    fn read(&self, addr: Address) -> Byte;
    fn write(&mut self, addr: Address, val: Byte);
}

trait PersistentMemory {
    fn save(file: impl Into<PathBuf>) -> Result<(), io::Error>;
    fn load(file: impl Into<PathBuf>) -> Result<(), io::Error>;
}
