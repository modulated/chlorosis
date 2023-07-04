use crate::Device;

use super::opcodes::Opcode;

impl Device {
    pub fn fetch_instruction(&mut self) -> Opcode {
        use Opcode::*;
        let op = self.consume_byte();

        match op.0 {
            // Row 0
            0x00 => NOP,
            0x01 => LD_BC_d16(self.consume_pair()),
            0x02 => LD_aBC_A,
            0x03 => INC_BC,
            0x04 => INC_B,
            0x05 => DEC_B,
            0x06 => LD_B_d8(self.consume_byte()),
            0x07 => RLCA,
            0x08 => LD_a16_SP(self.consume_pair()),
            0x09 => ADD_HL_BC,
            0x0A => LD_A_aBC,
            0x0B => DEC_BC,
            0x0C => INC_C,
            0x0D => DEC_C,
            0x0E => LD_C_d8(self.consume_byte()),
            0x0F => RRCA,
            // Row 0

            // Row 1
            0x10 => STOP(self.consume_byte()),
            0x11 => LD_DE_d16(self.consume_pair()),
            0x12 => LD_aDE_A,
            0x13 => INC_DE,
            0x14 => INC_D,
            0x15 => DEC_D,
            0x16 => LD_D_d8(self.consume_byte()),
            0x17 => RLA,
            0x18 => JR_s8(self.consume_signed_byte()),
            0x19 => ADD_HL_DE,
            0x1A => LD_A_aDE,
            0x1B => DEC_DE,
            0x1C => INC_E,
            0x1D => DEC_D,
            0x1E => LD_E_d8(self.consume_byte()),
            0x1F => RRA,
            // Row 1

            // Row 2
            0x20 => JR_NZ_s8(self.consume_signed_byte()),
            0x21 => LD_HL_d16(self.consume_pair()),
            0x22 => LD_aHL_inc_A,
            0x23 => INC_HL,
            0x24 => INC_H,
            0x25 => DEC_H,
            0x26 => LD_H_d8(self.consume_byte()),
            0x27 => DAA,
            0x28 => JR_Z_s8(self.consume_signed_byte()),
            0x29 => ADD_HL_HL,
            0x2A => LD_A_aHL_inc,
            0x2B => DEC_HL,
            0x2C => INC_L,
            0x2D => DEC_L,
            0x2E => LD_L_d8(self.consume_byte()),
            0x2F => CPL,
            // Row 2

            // Row 3
            0x30 => JR_NC_s8(self.consume_signed_byte()),
            0x31 => LD_SP_d16(self.consume_pair()),
            0x32 => LD_aHL_dec_A,
            0x33 => INC_SP,
            0x34 => INC_aHL,
            0x35 => DEC_aHL,
            0x36 => LD_aHL_d8(self.consume_byte()),
            0x37 => SCF,
            0x38 => JR_C_s8(self.consume_signed_byte()),
            0x39 => ADD_HL_SP,
            0x3A => LD_A_aHL_dec,
            0x3B => DEC_SP,
            0x3C => INC_A,
            0x3D => DEC_A,
            0x3E => LD_A_d8(self.consume_byte()),
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

            // Row 8
            0x80 => ADD_B,
            0x81 => ADD_C,
            0x82 => ADD_D,
            0x83 => ADD_E,
            0x84 => ADD_H,
            0x85 => ADD_L,
            0x86 => ADD_aHL,
            0x87 => ADD_A,
            0x88 => ADC_B,
            0x89 => ADC_C,
            0x8A => ADC_D,
            0x8B => ADC_E,
            0x8C => ADC_H,
            0x8D => ADC_L,
            0x8E => ADC_aHL,
            0x8F => ADC_A,
            // Row 8

            // Row 9
            0x90 => SUB_B,
            0x91 => SUB_C,
            0x92 => SUB_D,
            0x93 => SUB_E,
            0x94 => SUB_H,
            0x95 => SUB_L,
            0x96 => SUB_aHL,
            0x97 => SUB_A,
            0x98 => SBC_B,
            0x99 => SBC_C,
            0x9A => SBC_D,
            0x9B => SBC_E,
            0x9C => SBC_H,
            0x9D => SBC_L,
            0x9E => SBC_aHL,
            0x9F => SBC_A,
            // Row 9

            // Row A
            0xA0 => AND_B,
            0xA1 => AND_C,
            0xA2 => AND_D,
            0xA3 => AND_E,
            0xA4 => AND_H,
            0xA5 => AND_L,
            0xA6 => AND_aHL,
            0xA7 => AND_A,
            0xA8 => XOR_B,
            0xA9 => XOR_C,
            0xAA => XOR_D,
            0xAB => XOR_E,
            0xAC => XOR_H,
            0xAD => XOR_L,
            0xAE => XOR_aHL,
            0xAF => XOR_A,
            // Row A

            // Row B
            0xB0 => OR_B,
            0xB1 => OR_C,
            0xB2 => OR_D,
            0xB3 => OR_E,
            0xB4 => OR_H,
            0xB5 => OR_L,
            0xB6 => OR_aHL,
            0xB7 => OR_A,
            0xB8 => CP_B,
            0xB9 => CP_C,
            0xBA => CP_D,
            0xBB => CP_E,
            0xBC => CP_H,
            0xBD => CP_L,
            0xBE => CP_aHL,
            0xBF => CP_A,
            // Row B

            // Row C
            0xC0 => RET_NZ,
            0xC1 => POP_BC,
            0xC2 => JP_NZ_a16(self.consume_pair()),
            0xC3 => JP_a16(self.consume_pair()),
            0xC4 => CALL_NZ_a16(self.consume_pair()),
            0xC5 => PUSH_BC,
            0xC6 => ADD_A_d8(self.consume_byte()),
            0xC7 => RST_0,
            0xC8 => RET_Z,
            0xC9 => RET,
            0xCA => JP_Z_a16(self.consume_pair()),
            0xCB => self.fetch_cb_instruction(),
            0xCC => CALL_Z_a16(self.consume_pair()),
            0xCD => CALL_a16(self.consume_pair()),
            0xCE => ADC_A_d8(self.consume_byte()),
            0xCF => RST_1,
            // Row C

            // Row D
            0xD0 => RET_NC,
            0xD1 => POP_DE,
            0xD2 => JP_NC_a16(self.consume_pair()),
            0xD3 => panic!("Illegal instruction {}", op),
            0xD4 => CALL_NC_a16(self.consume_pair()),
            0xD5 => PUSH_DE,
            0xD6 => SUB_d8(self.consume_byte()),
            0xD7 => RST_2,
            0xD8 => RET_C,
            0xD9 => RETI,
            0xDA => JP_C_a16(self.consume_pair()),
            0xDB => panic!("Illegal instruction {}", op),
            0xDC => CALL_C_a16(self.consume_pair()),
            0xDD => panic!("Illegal instruction {}", op),
            0xDE => SBC_A_d8(self.consume_byte()),
            0xDF => RST_3,
            // Row D

            // Row E
            0xE0 => LD_a8_A(self.consume_byte().to_address()),
            0xE1 => POP_HL,
            0xE2 => LD_aC_A,
            0xE3 => panic!("Illegal instruction {}", op),
            0xE4 => panic!("Illegal instruction {}", op),
            0xE5 => PUSH_HL,
            0xE6 => AND_d8(self.consume_byte()),
            0xE7 => RST_4,
            0xE8 => ADD_SP_s8(self.consume_signed_byte()),
            0xE9 => JP_HL,
            0xEA => LD_a16_A(self.consume_pair()),
            0xEB => panic!("Illegal instruction {}", op),
            0xEC => panic!("Illegal instruction {}", op),
            0xED => panic!("Illegal instruction {}", op),
            0xEE => XOR_d8(self.consume_byte()),
            0xEF => RST_5,
            // Row E

            // Row F
            0xF0 => LD_A_a8(self.consume_byte().to_address()),
            0xF1 => POP_AF,
            0xF2 => LD_A_aC,
            0xF3 => DI,
            0xF4 => panic!("Illegal instruction {}", op),
            0xF5 => PUSH_AF,
            0xF6 => OR_d8(self.consume_byte()),
            0xF7 => RST_6,
            0xF8 => LD_HL_SP_s8(self.consume_signed_byte()),
            0xF9 => LD_SP_HL,
            0xFA => LD_A_a16(self.consume_pair()),
            0xFB => EI,
            0xFC => panic!("Illegal instruction {}", op),
            0xFD => panic!("Illegal instruction {}", op),
            0xFE => CP_d8(self.consume_byte()),
            0xFF => RST_7,
            // Row F

            // _ => unreachable!("Unknown Opcode 0x{} at address {}", op, current_address),
        }
    }

    pub fn fetch_cb_instruction(&mut self) -> Opcode {
        use Opcode::*;
        let current_address = self.cpu.pc;
        let op = self.consume_byte();

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
