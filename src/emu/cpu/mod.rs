use super::{memory::MemoryMap, types::SignedByte, Address, Byte};

mod arith;
mod execute;
mod fetch;
mod macros;
mod opcodes;

#[derive(Debug)]
pub struct CentralProcessor {
    a: Byte,
    // f: Byte,
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
    cycle_count: u64,
}

impl Default for CentralProcessor {
    fn default() -> Self {
        Self {
            a: Byte(0x00),
            b: Byte(0x00),
            c: Byte(0x00),
            d: Byte(0x00),
            e: Byte(0x00),
            h: Byte(0x00),
            l: Byte(0x00),
            z_flag: false,
            n_flag: false,
            h_flag: false,
            c_flag: false,
            pc: Address(0x0100),
            sp: Address(0xFFFE),
            cost: 0,
            cycle_count: 0,
            interupt_master_enable: false,
        }
    }
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
        println!("{op:?}");

        // execute instruction
        self.execute(mmap, op);

        self.dump_state();
    }

    pub fn read_f(&self) -> Byte {
        // TODO: may be able to make const?
        let mut b = Byte(0x0);
        b.write_bit(7, self.z_flag);
        b.write_bit(6, self.n_flag);
        b.write_bit(5, self.h_flag);
        b.write_bit(4, self.c_flag);
        b
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

    pub fn read_af(&self) -> Address {
        Address(((self.a.0 as u16) << 8) + self.read_f().0 as u16)
    }

    pub fn consume_byte(&mut self, mmap: &mut MemoryMap) -> Byte {
        let out = mmap.read(self.pc);
        self.pc += 1;
        out
    }

    fn consume_signed_byte(&mut self, mmap: &mut MemoryMap) -> SignedByte {
        let out = mmap.read(self.pc);
        self.pc += 1;
        out.to_signed()
    }

    fn consume_pair(&mut self, mmap: &mut MemoryMap) -> Address {
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
            "A: {} B: {} C: {} D: {} E: {} H: {} L: {}",
            self.a, self.b, self.c, self.d, self.e, self.h, self.l
        );
        println!(
            "Z: {} N: {} H: {} C: {}",
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

    fn write_f(&mut self, val: Byte) {
        self.z_flag = val.is_bit_set(7);
        self.n_flag = val.is_bit_set(6);
        self.h_flag = val.is_bit_set(5);
        self.c_flag = val.is_bit_set(4);
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

    fn write_af(&mut self, addr: Address) {
        let (a, f) = addr.split();
        self.a = a;
        self.write_f(f);
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

    #[test]
    fn test_carry_add_byte() {
        let mut cpu = CentralProcessor::new();
        cpu.check_carry_add_byte(Byte(0x80), Byte(0x80));
        assert!(cpu.c_flag);
        cpu.check_carry_add_byte(Byte(0x0F), Byte(0x70));
        assert!(!cpu.c_flag);
    }

    fn test_half_carry_add_byte() {
        let mut cpu = CentralProcessor::new();
        cpu.check_half_carry_add_byte(Byte(0x08), Byte(0x08));
        assert!(cpu.h_flag);
        cpu.check_half_carry_add_byte(Byte(0x04), Byte(0x10));
        assert!(!cpu.h_flag);
        cpu.check_half_carry_add_byte(Byte(0x08), Byte(0x01));
        assert!(!cpu.h_flag);
    }

    fn test_half_carry_sub_byte() {
        let mut cpu = CentralProcessor::new();
        cpu.check_half_carry_sub_byte(Byte(0x01), Byte(0x00));
        assert!(cpu.h_flag);
        cpu.check_half_carry_sub_byte(Byte(0x14), Byte(0x10));
        assert!(!cpu.h_flag);
        cpu.check_half_carry_sub_byte(Byte(0x08), Byte(0x01));
        assert!(!cpu.h_flag);
    }
}
