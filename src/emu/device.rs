use super::{AudioProcessor, CentralProcessor, MemoryMap, PixelProcessor, Screen};

#[derive(Debug, Default)]
pub struct Device {
    mmap: MemoryMap,
    cpu: CentralProcessor,
    ppu: PixelProcessor,
    audio: AudioProcessor,
    screen: Screen,
}

impl Device {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(&mut self) {
        loop {
            //
            self.cpu.cycle(&mut self.mmap);
        }
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

    pub fn load_boot(&mut self) {
        let boot = include_bytes!("../../cgb_boot.bin");
        self.mmap.load_cartrige(boot.to_vec());
    }

    pub fn dump_memory(&mut self) {
        self.mmap.dump_cartrige();
    }

    pub fn dump_state(&self) {
        println!("CPU State: ");
        self.cpu.dump_state();
    }
}

#[cfg(test)]
mod test {
    use super::Device;
    #[test]
    fn test_load_file() {
        let mut dev = Device::new();
        dev.load_from_file("cgb_boot.bin")
            .unwrap();
    }
}
