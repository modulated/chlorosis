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
    z_flag: bool,
    n_flag: bool,
    h_flag: bool,
    c_flag: bool,
    pc: Address,
    sp: Address,
    interupt_master_enable: bool,
    cost: u8,
    cycle_count: u64
}

impl CentralProcessor {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn cycle(&mut self, mmap: &mut MemoryMap) {
        // return if cycle timer not 0
        if self.cost != 0 {
            self.cost -= 1;
            // println!("Tick");
            return;
        }
        // fetch instruction
        let op = self.fetch_instruction(mmap);
        // println!("{op:?}");

        // execute instruction
        self.execute(mmap, op);

        // self.dump_state();
        self.cycle_count += 1;
        if self.cycle_count % 1_000_000 == 0 {
            println!("1MHz");
        }
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

    fn consume_pair_be(&mut self, mmap: &mut MemoryMap) -> Address {
        let b1 = mmap.read(self.pc);
        self.pc += 1;
        let b2 = mmap.read(self.pc);
        self.pc += 1;
        Address(((b1.0 as u16) << 8) + b2.0 as u16)
    }

    fn consume_pair_le(&mut self, mmap: &mut MemoryMap) -> Address {
        let b1 = mmap.read(self.pc);
        self.pc += 1;
        let b2 = mmap.read(self.pc);
        self.pc += 1;
        Address(((b2.0 as u16) << 8) + b1.0 as u16)
    }

    pub fn dump_state(&self) {
        println!("Cost: {}", self.cost);
        println!("PC: {} SP: {}", self.pc, self.sp);
        println!(
            "A: {} F: {} B: {} C: {} D: {} E: {} H: {} L: {}",
            self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l
        );
        println!(
            "Flags- Z: {} N: {} H: {} C: {}",
            self.z_flag, self.n_flag, self.h_flag, self.c_flag
        );
        println!();
    }

    fn push_address(&mut self, mmap: &mut MemoryMap, addr: Address) {
        let (h, l) = addr.split();
        self.sp -= 1;
        mmap.write(self.sp, h);
        self.sp -= 1;
        mmap.write(self.sp, l);
    }

    fn pop_address(&mut self, mmap: &mut MemoryMap) -> Address {
        let l = mmap.read(self.sp);
        self.sp += 1;
        let h = mmap.read(self.sp);
        self.sp += 1;
        Address::from_pair(h, l)
    }

    fn clear_flags(&mut self) {
        self.z_flag = false;
        self.n_flag = false;
        self.h_flag = false;
        self.c_flag = false;
    }

    fn write_bc(&mut self, addr: Address) {
        let (b, c) = addr.split();
        self.b = b;
        self.c = c;
    }

    fn write_de(&mut self, addr: Address) {
        let (d, e) = addr.split();
        self.d = d;
        self.e = e;
    }

    fn write_hl(&mut self, addr: Address) {
        let (h, l) = addr.split();
        self.h = h;
        self.l = l;
    }
}

#[cfg(test)]
mod test {
    use super::CentralProcessor;
    use crate::emu::{Address, Byte, MemoryMap};

    #[test]
    fn test_register_combining() {
        let mut cpu = CentralProcessor::new();
        cpu.b = Byte(0xFA);
        cpu.c = Byte(0xCE);
        let addr = cpu.read_bc();
        assert_eq!(addr, Address(0xFACE));
    }

    #[test]
    fn test_stack_push() {
        let mut cpu = CentralProcessor::new();
        cpu.sp = Address(0xFFFE);
        let mut mmap = MemoryMap::new();

        cpu.push_address(&mut mmap, Address(0x1234));
        cpu.push_address(&mut mmap, Address(0x5678));
        cpu.push_address(&mut mmap, Address(0x9ABC));

        assert_eq!(cpu.pop_address(&mut mmap), Address(0x9ABC));
        assert_eq!(cpu.pop_address(&mut mmap), Address(0x5678));
        assert_eq!(cpu.pop_address(&mut mmap), Address(0x1234));
    }
}
