use super::{memory::MemoryMap, Address, Byte};

mod execute;
mod fetch;
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
    cycle_timer: u8,
}

impl CentralProcessor {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn cycle(&mut self, mmap: &mut MemoryMap) {
        // return if cycle timer not 0
        if self.cycle_timer != 0 {
            self.cycle_timer -= 1;
            println!("Tick");
            return;
        }
        // fetch instruction
        let op = self.fetch_instruction(mmap);
        self.dump_state();
        println!("{op:?}");

        self.execute(mmap, op);
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

    pub fn consume_byte(&mut self, mmap: &mut MemoryMap) -> Byte {
        let out = mmap.read(self.pc);
        self.pc += 1u8;
        out
    }

    fn consume_pair(&mut self, mmap: &mut MemoryMap) -> Address {
        let b1 = mmap.read(self.pc);
        self.pc += 1;
        let b2 = mmap.read(self.pc);
        self.pc += 1;
        Address(((b1.0 as u16) << 8) + b2.0 as u16)
    }

    pub fn dump_state(&self) {
        println!("Cycle timer: {}", self.cycle_timer);
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
