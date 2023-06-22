use crate::emu::MemoryMap;

use super::{opcodes::Opcode, CentralProcessor};

impl CentralProcessor {
    pub fn fetch_instruction(&mut self, mmap: &mut MemoryMap) -> Opcode {
        use Opcode::*;
        let current_address = self.pc;
        let op = self.consume_byte(mmap);

        match op.0 {
            0x00 => NOP,

            0x0D => DEC_C,
            0x0E => LD_C_d8(self.consume_byte(mmap)),

            0x21 => LD_HL_d16(self.consume_byte(mmap), self.consume_byte(mmap)),
            0x22 => LD_HL_inc_A,

            0x2F => CPL,

            0x31 => LD_SP_d16(self.consume_pair_le(mmap)),

            0x3E => LD_A_d8(self.consume_byte(mmap)),

            0xC2 => JP_NZ_a16(self.consume_pair_le(mmap)),
            0xC3 => JP_a16(self.consume_pair_le(mmap)),

            0xCD => CALL_a16(self.consume_pair_le(mmap)),

            0xAF => XOR_A,

            0xE0 => LD_a8_A(self.consume_byte(mmap).to_address()),

            _ => panic!(
                "Unimplemented Opcode 0x{} at address {}",
                op, current_address
            ),
        }
    }
}
