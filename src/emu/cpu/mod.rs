use super::{memory::MemoryMap, Address, Byte};

mod opcodes;

#[derive(Debug, Default)]
pub struct CentralProcessor {
    a: Byte,
    f: Byte,
    b: Byte,
    c: Byte,
    d: Byte,
    e: Byte,
    h: Byte,
    l: Byte,
    pc: Address,
    sp: Address,
    interupt_master_enable: bool,
}

impl CentralProcessor {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn cycle(&mut self, _mmap: &mut MemoryMap) {
        // fetch
        // decode
        // execute
        unimplemented!()
    }

    pub const fn read_bc(&self) -> Address {
        Address(((self.b.0 as u16) << 8) + self.c.0 as u16)
    }

    pub const fn read_de(&self) -> Address {
        Address(((self.d.0 as u16) << 8) + self.e.0 as u16)
    }

    pub const fn read_hl(&self) -> Address {
        Address(((self.h.0 as u16) << 8) + self.l.0 as u16)
    }

    pub fn dump_state(&self) {
        println!("PC: {} SP: {}", self.pc, self.sp);
        println!("A: {} F: {}", self.a, self.f);
        println!("B: {} C: {}", self.b, self.c);
        println!("D: {} E: {}", self.d, self.e);
        println!("H: {} L: {}", self.h, self.l);
    }
}

#[cfg(test)]
mod test {
    use super::CentralProcessor;
    use crate::emu::{Address, Byte};

    #[test]
    fn test_register_combining() {
        let mut cpu = CentralProcessor::new();
        cpu.b = Byte(0xFA);
        cpu.c = Byte(0xCE);
        let addr = cpu.read_bc();
        assert_eq!(addr, Address(0xFACE));
    }
}
