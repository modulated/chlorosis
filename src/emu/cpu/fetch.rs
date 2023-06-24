use crate::emu::MemoryMap;

use super::{opcodes::Opcode, CentralProcessor};

impl CentralProcessor {
    pub fn fetch_instruction(&mut self, mmap: &mut MemoryMap) -> Opcode {
        use Opcode::*;
        let current_address = self.pc;
        let op = self.consume_byte(mmap);

        match op.0 {
            // Row 0
            0x00 => NOP,
            0x01 => LD_BC_d16(self.consume_pair(mmap)),
            0x02 => LD_aBC_A,
            0x03 => INC_BC,
            0x04 => INC_B,
            0x05 => DEC_B,
            0x06 => LD_B_d8(self.consume_byte(mmap)),
            0x07 => RLCA,
            0x08 => LD_a16_SP(self.consume_pair(mmap)),
            0x09 => ADD_HL_BC,
            0x0A => LD_A_aBC,
            0x0B => DEC_BC,
            0x0C => INC_C,
            0x0D => DEC_C,
            0x0E => LD_C_d8(self.consume_byte(mmap)),
            0x0F => RRCA,
            // Row 0

            // Row 1
            0x10 => STOP(self.consume_byte(mmap)),
            0x11 => LD_DE_d16(self.consume_pair(mmap)),
            0x12 => LD_aDE_A,
            0x13 => INC_DE,
            0x14 => INC_D,
            0x15 => DEC_D,
            0x16 => LD_D_d8(self.consume_byte(mmap)),
            0x17 => RLA,
            0x18 => JR_s8(self.consume_signed_byte(mmap)),
            0x19 => ADD_HL_DE,
            0x1A => LD_A_aDE,
            0x1B => DEC_DE,
            0x1C => INC_E,
            0x1D => DEC_D,
            0x1E => LD_E_d8(self.consume_byte(mmap)),
            0x1F => RRA,
            // Row 1

            // Row 2
            0x20 => JR_NZ_s8(self.consume_signed_byte(mmap)),
            0x21 => LD_HL_d16(self.consume_byte(mmap), self.consume_byte(mmap)),
            0x22 => LD_aHL_inc_A,
            0x23 => INC_HL,
            0x24 => INC_H,
            0x25 => DEC_H,
            0x26 => LD_H_d8(self.consume_byte(mmap)),
            0x27 => DAA,
            0x28 => JR_Z_s8(self.consume_signed_byte(mmap)),
            0x29 => ADD_HL_HL,
            0x2A => LD_A_aHL_inc,
            0x2B => DEC_HL,
            0x2C => INC_L,
            0x2D => DEC_L,
            0x2E => LD_L_d8(self.consume_byte(mmap)),
            0x2F => CPL,
            // Row 2

            // Row 3
            0x30 => JR_NC_s8(self.consume_signed_byte(mmap)),
            0x31 => LD_SP_d16(self.consume_pair(mmap)),
            0x32 => LD_HL_dec_A,
            0x33 => INC_SP,
            0x34 => INC_aHL,
            0x35 => DEC_aHL,
            0x36 => LD_aHL_d8(self.consume_byte(mmap)),
            0x37 => SCF,
            0x38 => JR_C_s8(self.consume_signed_byte(mmap)),
            0x39 => ADD_HL_SP,
            0x3A => LD_A_aHL_dec,
            0x3B => DEC_SP,
            0x3C => INC_A,
            0x3D => DEC_A,
            0x3E => LD_A_d8(self.consume_byte(mmap)),
            0x3F => CCF,
            // Row 3

            // Row 4
            0x4F => LD_C_A,

            0x77 => LD_HL_A,

            0xAF => XOR_A,

            0xBC => CP_H,

            0xC1 => POP_BC,
            0xC2 => JP_NZ_a16(self.consume_pair(mmap)),
            0xC3 => JP_a16(self.consume_pair(mmap)),
            0xC4 => CALL_NZ_a16(self.consume_pair(mmap)),
            0xC5 => PUSH_BC,

            0xCB => self.fetch_cb_instruction(mmap),
            0xCD => CALL_a16(self.consume_pair(mmap)),

            0xE0 => LD_a8_A(self.consume_byte(mmap).to_address()),
            0xE1 => POP_HL,
            0xE2 => LD_aC_A,

            _ => panic!("Unknown Opcode 0x{} at address {}", op, current_address),
        }
    }

    pub fn fetch_cb_instruction(&mut self, mmap: &mut MemoryMap) -> Opcode {
        use Opcode::*;
        let current_address = self.pc;
        let op = self.consume_byte(mmap);

        match op.0 {
            0x11 => RL_C,

            0x7C => BIT_7_H,
            _ => panic!(
                "Unknown CB Opcode 0xCB{} at address {} + {}",
                op,
                current_address - 1,
                current_address
            ),
        }
    }
}
