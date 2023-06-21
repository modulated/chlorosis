use crate::emu::MemoryMap;

use super::{opcodes::Opcode, CentralProcessor};

impl CentralProcessor {
    pub fn fetch_instruction(&mut self, mmap: &mut MemoryMap) -> Opcode {
        use Opcode::*;
        let op = self.consume_byte(mmap);

        match op.0 {
            0x00 => NOP,

            0x31 => LD_SP_d16(self.consume_pair(mmap)),

            0x3E => LD_A_d8(self.consume_byte(mmap)),

            _ => panic!("Unimplemented opcode 0x{}", op),
        }
    }
}
