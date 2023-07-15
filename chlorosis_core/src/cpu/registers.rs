use crate::{CentralProcessor, types::Byte};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L, 
}

impl CentralProcessor {
    pub fn read_register(&self, reg: Register) -> Byte {
        match reg {
            Register::A => self.a,
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::H => self.h,
            Register::L => self.l
        }
    }

    pub fn write_register(&mut self, reg: Register, val: Byte) {
        match reg {
            Register::A => self.a = val,
            Register::B => self.b = val,
            Register::C => self.c = val,
            Register::D => self.d = val,
            Register::E => self.e = val,
            Register::H => self.h = val,
            Register::L => self.l = val,
        }
    }
}