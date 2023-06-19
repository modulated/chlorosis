use super::{audio::AudioProcessor, cpu::CentralProcessor, memory::MemoryMap, ppu::PixelProcessor};

#[derive(Debug, Default)]
pub struct Device {
    mmap: MemoryMap,
    cpu: CentralProcessor,
    ppu: PixelProcessor,
    audio: AudioProcessor,
}

impl Device {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn load_from_file(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), std::io::Error> {
        use std::io::Read;
        let mut f = std::fs::File::open(path)?;
        let mut buf = vec![];
        f.read_to_end(&mut buf)?;
        self.mmap.load_cartrige(buf);

        Ok(())
    }

    pub fn dump_memory(&mut self) {
        self.mmap.dump_cartrige();
    }
}

#[cfg(test)]
mod test {
    use super::Device;
    #[test]
    fn test_load_file() {
        let mut dev = Device::new();
        dev.load_from_file("test_roms/little-things-gb/firstwhite.gb")
            .unwrap();
    }
}
