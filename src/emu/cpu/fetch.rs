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
            0x21 => LD_HL_d16(self.consume_pair(mmap)),
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
            0x32 => LD_aHL_dec_A,
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
            0x40 => LD_B_B,
            0x41 => LD_B_C,
            0x42 => LD_B_D,
            0x43 => LD_B_E,
            0x44 => LD_B_H,
            0x45 => LD_B_L,
            0x46 => LD_B_aHL,
            0x47 => LD_B_A,
            0x48 => LD_C_B,
            0x49 => LD_C_C,
            0x4A => LD_C_D,
            0x4B => LD_C_E,
            0x4C => LD_C_H,
            0x4D => LD_C_L,
            0x4E => LD_C_aHL,
            0x4F => LD_C_A,
            // Row 4

            // Row 5
            0x50 => LD_D_B,
            0x51 => LD_D_C,
            0x52 => LD_D_D,
            0x53 => LD_D_E,
            0x54 => LD_D_H,
            0x55 => LD_D_L,
            0x56 => LD_D_aHL,
            0x57 => LD_D_A,
            0x58 => LD_E_B,
            0x59 => LD_E_C,
            0x5A => LD_E_D,
            0x5B => LD_E_E,
            0x5C => LD_E_H,
            0x5D => LD_E_L,
            0x5E => LD_E_aHL,
            0x5F => LD_E_A,
            // Row 5

            // Row 6
            0x60 => LD_H_B,
            0x61 => LD_H_C,
            0x62 => LD_H_D,
            0x63 => LD_H_E,
            0x64 => LD_H_H,
            0x65 => LD_H_L,
            0x66 => LD_H_aHL,
            0x67 => LD_H_A,
            0x68 => LD_L_B,
            0x69 => LD_L_C,
            0x6A => LD_L_D,
            0x6B => LD_L_E,
            0x6C => LD_L_H,
            0x6D => LD_L_L,
            0x6E => LD_L_aHL,
            0x6F => LD_L_A,
            // Row 6

            // Row 7
            0x70 => LD_aHL_B,
            0x71 => LD_aHL_C,
            0x72 => LD_aHL_D,
            0x73 => LD_aHL_E,
            0x74 => LD_aHL_H,
            0x75 => LD_aHL_L,
            0x76 => HALT,
            0x77 => LD_aHL_A,
            0x78 => LD_A_B,
            0x79 => LD_A_C,
            0x7A => LD_A_D,
            0x7B => LD_A_E,
            0x7C => LD_A_H,
            0x7D => LD_A_L,
            0x7E => LD_A_aHL,
            0x7F => LD_A_A,
            // Row 7
            0x90 => SUB_B,

            0xAF => XOR_A,

            0xBC => CP_H,

            0xC1 => POP_BC,
            0xC2 => JP_NZ_a16(self.consume_pair(mmap)),
            0xC3 => JP_a16(self.consume_pair(mmap)),
            0xC4 => CALL_NZ_a16(self.consume_pair(mmap)),
            0xC5 => PUSH_BC,

            0xC9 => RET,

            0xCB => self.fetch_cb_instruction(mmap),
            0xCD => CALL_a16(self.consume_pair(mmap)),

            0xE0 => LD_a8_A(self.consume_byte(mmap).to_address()),
            0xE1 => POP_HL,
            0xE2 => LD_aC_A,

            0xEA => LD_a16_A(self.consume_pair(mmap)),

            0xF0 => LD_A_a8(self.consume_byte(mmap)),

            0xFE => CP_d8(self.consume_byte(mmap)),

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
