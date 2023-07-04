use crate::Device;

use super::{types::SignedByte, Address, Byte};

mod arith;
mod execute;
mod fetch;
mod macros;
mod opcodes;

#[derive(Debug)]
pub struct CentralProcessor {
    pub a: Byte,
    pub b: Byte,
    pub c: Byte,
    pub d: Byte,
    pub e: Byte,
    pub h: Byte,
    pub l: Byte,
    pub z_flag: bool,
    pub n_flag: bool,
    pub h_flag: bool,
    pub c_flag: bool,
    pub pc: Address,
    pub sp: Address,
    pub interupt_master_enable: bool,
    pub cost: u8
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
            interupt_master_enable: false,
            // cycle_count: 0,
        }
    }
}

impl CentralProcessor {
    pub fn new() -> Self {
        Default::default()
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
}

impl Device {
    pub fn step_cpu(&mut self) {
        // return if cycle timer not 0
        if self.cpu.cost != 0 {
            self.cpu.cost -= 1;
            return;
        }
        // fetch instruction
        let op = self.fetch_instruction();

        // execute instruction
        self.execute(op);
    }



    pub fn consume_byte(&mut self) -> Byte {
        let out = self.read(self.cpu.pc);
        self.cpu.pc += 1;
        out
    }

    fn consume_signed_byte(&mut self) -> SignedByte {
        let out = self.read(self.cpu.pc);
        self.cpu.pc += 1;
        out.to_signed()
    }

    fn consume_pair(&mut self) -> Address {
        let b1 = self.read(self.cpu.pc);
        self.cpu.pc += 1;
        let b2 = self.read(self.cpu.pc);
        self.cpu.pc += 1;
        Address(((b2.0 as u16) << 8) + b1.0 as u16)
    }

    fn push_address(&mut self, addr: Address) {
        let (h, l) = addr.split();
        self.cpu.sp -= 1;
        self.write(self.cpu.sp, h);
        self.cpu.sp -= 1;
        self.write(self.cpu.sp, l);
    }

    fn pop_address(&mut self) -> Address {
        let l = self.read(self.cpu.sp);
        self.cpu.sp += 1;
        let h = self.read(self.cpu.sp);
        self.cpu.sp += 1;
        Address::from_pair(h, l)
    }
}
