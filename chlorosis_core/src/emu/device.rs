use super::{
    types::CartrigeHeader, AudioProcessor, CentralProcessor, MemoryMap, PixelProcessor, Screen,
};

#[derive(Debug, Default)]
pub struct Device {
    pub mmap: MemoryMap,
    pub cpu: CentralProcessor,
    pub ppu: PixelProcessor,
    audio: AudioProcessor,
    screen: Screen,
    cartrige: Option<CartrigeHeader>,
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

    pub fn load_cartrige(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), std::io::Error> {
        use std::io::Read;
        let mut f = std::fs::File::open(path)?;
        let mut buf = vec![];
        f.read_to_end(&mut buf)?;
        self.mmap.load_cartrige(buf);

        println!("{:?}", self.mmap.get_header());
        self.cartrige = Some(CartrigeHeader::from_bytes(self.mmap.get_header()));

        Ok(())
    }

    // pub fn dump_memory(&mut self) {
    //     self.mmap.dump_rom();
    // }

    pub fn dump_cpu(&self) {
        println!("CPU State: ");
        self.cpu.dump_state();
    }

    pub fn dump_cartrige_header(&self) {
        self.cartrige
            .as_ref()
            .map_or_else(|| println!("No cartrige loaded"), |c| println!("{c:#?}"))
    }
}
